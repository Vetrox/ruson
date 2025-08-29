use crate::nodes::node::{Node, NodeKind};
use crate::services::parser::{Parser, SCOPE_NID};
use crate::typ::typ::Typ;

pub fn as_dotfile(parser: &Parser) -> String {
    let mut sb = String::new();
    sb.push_str("digraph mygraph{\n");
    sb.push_str("/*\n");
    sb.push_str(&parser.src());
    sb.push_str("\n*/\n");

    // To keep the Scopes below the graph and pointing up into the graph we
    // need to group the Nodes in a subgraph cluster, and the scopes into a
    // different subgraph cluster.  THEN we can draw edges between the
    // scopes and nodes.  If we try to cross subgraph cluster borders while
    // still making the subgraphs DOT gets confused.

    sb.push_str("\trankdir=BT;\n"); // Force Nodes before Scopes

    // Preserve node input order
    sb.push_str("\tordering=\"in\";\n");

    // Merge multiple edges hitting the same node.  Makes common shared
    // nodes much prettier to look at.
    sb.push_str("\tconcentrate=\"true\";\n");

    // Just the Nodes first, in a cluster no edges
    sb.push_str("\tsubgraph cluster_Nodes {\n"); // Magic "cluster_" in the subgraph name

    let graph_br = parser.graph.borrow();

    // define normal nodes
    for n in graph_br.graph_iter().filter(|n| !matches!(n.node_kind, NodeKind::KeepAlive | NodeKind::Scope {..})) {
        sb.push_str("\t\t");
        sb.push_str(&format!("Node_{}", n.nid));
        sb.push_str(" [ ");
        let lab = node_icon(n);
        // control nodes have box shape
        // other nodes are ellipses, i.e. default shape
        if n.is_cfg() {
            sb.push_str("shape=box style=filled fillcolor=yellow ");
        }
        sb.push_str("label=\"");
        sb.push_str(&lab);
        sb.push_str("\" ");
        sb.push_str("];\n");
    }
    sb.push_str("\t}\n");     // End Node cluster

    // define the scope node
    if let NodeKind::Scope { scopes } = &graph_br.get_node(SCOPE_NID).unwrap().node_kind {
        sb.push_str("\tnode [shape=plaintext];\n");
        for (level, scope) in scopes.iter().enumerate() {
            sb.push_str("\tsubgraph cluster_");
            let scope_name = format!("Node_{}_{}", SCOPE_NID, level);
            sb.push_str(&scope_name);
            sb.push_str(" {\n\t\t");
            sb.push_str(&scope_name);
            sb.push_str(" [label=<\n\t\t\t<TABLE BORDER=\"0\" CELLBORDER=\"1\" CELLSPACING=\"0\">\n\t\t\t<TR><TD BGCOLOR=\"cyan\">");
            sb.push_str(&format!("{}", level));
            sb.push_str("</TD>");
            let mut keys: Vec<_> = scope.keys().collect();
            keys.sort();
            for name in keys {
                sb.push_str("<TD PORT=\"");
                sb.push_str(&format!("{}_{}", scope_name, name));
                sb.push_str("\">");
                sb.push_str(&format!("{}", name));
                sb.push_str("</TD>");
            }
            sb.push_str("</TR>\n\t\t\t</TABLE>>];\n");
        }
        sb.push_str(&"\t}\n".repeat(scopes.len()));
    }

    // Walk the Node edges
    sb.push_str("\tedge [ fontname=Helvetica, fontsize=8 ];\n");
    for n in graph_br.graph_iter().filter(|n| !matches!(n.node_kind, NodeKind::KeepAlive | NodeKind::Scope {..})) {
        // In this chapter we do display the Constant->Start edge;
        for (i, def_nid) in n.inputs.iter().enumerate() {
            if let Some(Some(def)) = graph_br.get(*def_nid) {
                // Most edges land here use->def
                sb.push('\t');
                sb.push_str(&format!("Node_{}", n.nid));
                sb.push_str(" -> ");
                sb.push_str(&format!("Node_{}", def_nid));
                // Number edges, so we can see how they track
                sb.push_str("[taillabel=");
                sb.push_str(&format!("{}", i));
                if matches!(n.node_kind, NodeKind::Constant {..}) && matches!(def.node_kind, NodeKind::Start {..}) {
                    sb.push_str(" style=dotted");
                } else if def.is_cfg() {   // control edges are colored red
                    sb.push_str(" color=red");
                }
                sb.push_str("];\n");
            }
        }
    }

    // Walk the variable definitions
    sb.push_str("\tedge [style=dashed color=cornflowerblue];\n");
    let scope_node = graph_br.get_node(SCOPE_NID).unwrap();
    if let NodeKind::Scope { scopes } = &scope_node.node_kind {
        for (level, scope) in scopes.iter().enumerate() {
            let scope_name = format!("Node_{}_{}", SCOPE_NID, level);
            for (name, def_nid) in scope {
                sb.push('\t');
                sb.push_str(&format!("{}:\"{}_{}\"", scope_name, scope_name, name));
                sb.push_str(" -> ");
                sb.push_str(&format!("Node_{}", def_nid));
                sb.push_str(";\n");
            }
        }
    }

    sb.push_str("}\n");
    sb
}

fn node_icon(node: &Node) -> String {
    match node.node_kind {
        NodeKind::Constant => {
            match node.typ() {
                Typ::Int { constant } => format!("#{}", constant),
                _ => panic!("Type {:?} for NodeKind::Constant unsupported", node.typ()),
            }
        }
        NodeKind::Return => "Return".into(),
        NodeKind::Start => "Start".into(),
        NodeKind::KeepAlive => "KeepAlive".into(),
        NodeKind::Add => "+".into(),
        NodeKind::Sub => "-".into(),
        NodeKind::Mul => "*".into(),
        NodeKind::Div => "/".into(),
        NodeKind::Minus => "-".into(),
        NodeKind::Scope { .. } => "Scope".into(),
    }
}

#[cfg(test)]
mod tests {
    use crate::services::dotvis::as_dotfile;
    use crate::services::parser::Parser;

    // #[test]
    fn should_output_minimal_dotfile() {
        // Arrange
        let parser = Parser::new("").unwrap();

        // Act
        let dotfile = as_dotfile(&parser);

        dbg!(&dotfile);

        // Assert
        assert_eq!(dotfile, "digraph mygraph{\n/*\n\n*/\n\trankdir=BT;\n\tordering=\"in\";\n\tconcentrate=\"true\";\n\tsubgraph cluster_Nodes {\n\t\tNode_1 [ shape=box style=filled fillcolor=yellow label=\"Start\" ];\n\t}\n\tedge [ fontname=Helvetica, fontsize=8 ];\n}\n");
    }

    // #[test]
    fn should_output_return_1_dotfile() {
        // Arrange
        let mut parser = Parser::new("return 1;").unwrap();
        parser.do_optimize = false;
        parser.parse().unwrap();

        // Act
        let dotfile = as_dotfile(&parser);

        dbg!(&dotfile);

        // Assert
        assert_eq!(dotfile, "digraph mygraph{\n/*\nreturn 1;\n*/\n\trankdir=BT;\n\tordering=\"in\";\n\tconcentrate=\"true\";\n\tsubgraph cluster_Nodes {\n\t\tNode_1 [ shape=box style=filled fillcolor=yellow label=\"Start\" ];\n\t\tNode_2 [ label=\"#1\" ];\n\t\tNode_3 [ shape=box style=filled fillcolor=yellow label=\"Return\" ];\n\t}\n\tedge [ fontname=Helvetica, fontsize=8 ];\n\tNode_3 -> Node_1[taillabel=0 color=red];\n\tNode_3 -> Node_2[taillabel=1];\n}\n");
    }

    // #[test]
    fn should_complex_dotfile() {
        // Arrange
        let mut parser = Parser::new("return 1+2*3+-5;").unwrap();
        parser.do_optimize = false;
        parser.parse().unwrap();

        // Act
        let dotfile = as_dotfile(&parser);

        dbg!(&dotfile);

        // Assert
        assert_eq!(dotfile, "digraph mygraph{\n/*\nreturn 1+2*3+-5;\n*/\n\trankdir=BT;\n\tordering=\"in\";\n\tconcentrate=\"true\";\n\tsubgraph cluster_Nodes {\n\t\tNode_1 [ shape=box style=filled fillcolor=yellow label=\"Start\" ];\n\t\tNode_2 [ label=\"#1\" ];\n\t\tNode_3 [ label=\"#2\" ];\n\t\tNode_4 [ label=\"#3\" ];\n\t\tNode_5 [ label=\"*\" ];\n\t\tNode_6 [ label=\"#5\" ];\n\t\tNode_7 [ label=\"-\" ];\n\t\tNode_8 [ label=\"+\" ];\n\t\tNode_9 [ label=\"+\" ];\n\t\tNode_10 [ shape=box style=filled fillcolor=yellow label=\"Return\" ];\n\t}\n\tedge [ fontname=Helvetica, fontsize=8 ];\n\tNode_5 -> Node_3[taillabel=0];\n\tNode_5 -> Node_4[taillabel=1];\n\tNode_7 -> Node_6[taillabel=0];\n\tNode_8 -> Node_5[taillabel=0];\n\tNode_8 -> Node_7[taillabel=1];\n\tNode_9 -> Node_2[taillabel=0];\n\tNode_9 -> Node_8[taillabel=1];\n\tNode_10 -> Node_1[taillabel=0 color=red];\n\tNode_10 -> Node_9[taillabel=1];\n}\n");
    }
}
