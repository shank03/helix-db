use std::collections::HashMap;

use crate::{protocol::value::Value, utils::items::Node};

pub struct VersionInfo(HashMap<String, ItemInfo>);

type NodeProps = HashMap<String, Value>;

struct ItemInfo {
    /// The latest version of this item
    /// All writes should be done with this version
    latest: u8,
    /// Stores transition from version x and index x-1
    transition_fns: Vec<fn(NodeProps) -> NodeProps>,
}

impl ItemInfo {
    fn upgrade_to_latest(&self, mut node: Node) -> Node {
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
}
