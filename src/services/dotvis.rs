use crate::nodes::node::{Node, NodeKind};
use crate::services::parser::Parser;

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
    let no_of_nodes = parser.graph.borrow().len();
    for nid in 0..no_of_nodes {
        if let Some(Some(n)) = parser.graph.borrow().get(nid) {
            sb.push_str("\t\t");
            sb.push_str(&format!("Node_{}", nid));
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
    }
    sb.push_str("\t}\n");     // End Node cluster

    // Walk the Node edges
    sb.push_str("\tedge [ fontname=Helvetica, fontsize=8 ];\n");
    let no_of_nodes = parser.graph.borrow().len();
    for nid in 0..no_of_nodes {
        if let Some(Some(n)) = parser.graph.borrow().get(nid) {
            // In this chapter we do display the Constant->Start edge;
            for (i, def_nid) in n.inputs.iter().enumerate() {
                //if (def != null) { TODO: currently we do NOT allow for Optionals in the inputs, which needs to be supported in the future.
                if let Some(Some(def)) = parser.graph.borrow().get(*def_nid) {
                    // Most edges land here use->def
                    sb.push('\t');
                    sb.push_str(&format!("Node_{}", nid));
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
    }

    sb.push_str("}\n");
    sb
}

fn node_icon(node: &Node) -> String {
    match node.node_kind {
        NodeKind::Constant { value } => format!("#{}", value),
        NodeKind::Return => "Return".into(),
        NodeKind::Start => "Start".into()
    }
}

#[cfg(test)]
mod tests {
    use crate::services::dotvis::as_dotfile;
    use crate::services::parser::Parser;

    #[test]
    fn should_output_minimal_dotfile() {
        // Arrange
        let parser = Parser::new("").unwrap();

        // Act
        let dotfile = as_dotfile(&parser);

        // Assert
        assert_eq!(dotfile, "digraph mygraph{\n/*\n\n*/\n\trankdir=BT;\n\tordering=\"in\";\n\tconcentrate=\"true\";\n\tsubgraph cluster_Nodes {\n\t\tNode_0 [ shape=box style=filled fillcolor=yellow label=\"Start\" ];\n\t}\n\tedge [ fontname=Helvetica, fontsize=8 ];\n}\n");
    }

    #[test]
    fn should_output_return_1_dotfile() {
        // Arrange
        let mut parser = Parser::new("return 1;").unwrap();
        parser.parse().unwrap();

        // Act
        let dotfile = as_dotfile(&parser);

        // Assert
        assert_eq!(dotfile, "digraph mygraph{\n/*\nreturn 1;\n*/\n\trankdir=BT;\n\tordering=\"in\";\n\tconcentrate=\"true\";\n\tsubgraph cluster_Nodes {\n\t\tNode_0 [ shape=box style=filled fillcolor=yellow label=\"Start\" ];\n\t\tNode_1 [ label=\"#1\" ];\n\t\tNode_2 [ shape=box style=filled fillcolor=yellow label=\"Return\" ];\n\t}\n\tedge [ fontname=Helvetica, fontsize=8 ];\n\tNode_1 -> Node_0[taillabel=0 style=dotted];\n\tNode_2 -> Node_0[taillabel=0 color=red];\n\tNode_2 -> Node_1[taillabel=1];\n}\n");
    }
}
