use std::collections::HashMap;

use crate::{
    protocol::value::Value,
    utils::items::{Edge, Node},
};

#[derive(Default, Clone)]
pub struct VersionInfo(pub HashMap<String, ItemInfo>);

impl VersionInfo {
    pub fn upgrade_to_node_latest(&self, node: Node) -> Node {
        let item_info = self
            .0
            .get(&node.label)
            .expect("All nodes should have version info");

        item_info.upgrade_node_to_latest(node)
    }

    pub fn upgrade_to_edge_latest(&self, node: Edge) -> Edge {
        let item_info = self
            .0
            .get(&node.label)
            .expect("All edges should have version info");

        item_info.upgrade_edge_to_latest(node)
    }

    pub fn get_latest(&self, label: &str) -> u8 {
        self.0
            .get(label)
            .expect("All labels should have version info")
            .latest
    }
}

type Props = HashMap<String, Value>;

#[derive(Clone)]
pub struct ItemInfo {
    /// The latest version of this item
    /// All writes should be done with this version
    latest: u8,
    /// Stores transition from version x and index x-1
    transition_fns: Vec<fn(Props) -> Props>,
}

impl ItemInfo {
    fn upgrade_node_to_latest(&self, mut node: Node) -> Node {
        if node.version < self.latest
            && let Some(mut node_props) = node.properties.take()
        {
            for trans_fn in self.transition_fns.iter().skip(node.version as usize - 1) {
                node_props = trans_fn(node_props);
            }

            node.properties = Some(node_props);
        }

        node
    }

    fn upgrade_edge_to_latest(&self, mut edge: Edge) -> Edge {
        if edge.version < self.latest
            && let Some(mut edge_props) = edge.properties.take()
        {
            for trans_fn in self.transition_fns.iter().skip(edge.version as usize - 1) {
                edge_props = trans_fn(edge_props);
            }

            edge.properties = Some(edge_props);
        }

        edge
    }
}
