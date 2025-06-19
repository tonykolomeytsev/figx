use json_event_parser::{JsonEvent, JsonParseError, ReaderJsonParser};
use std::{collections::VecDeque, fmt::Display, hash::Hasher, io::Read};

#[cfg_attr(test, derive(Debug, Eq, PartialEq, Hash))]
pub struct Node {
    pub id: String,
    pub name: String,
    pub visible: bool,
    pub has_raster_fills: bool,
    pub hash: u64,
}

pub struct NodeStream<R: Read> {
    reader: ReaderJsonParser<R>,
    stack: VecDeque<NodeDto>,
    state: NodeStreamState,
}

enum NodeStreamState {
    Default,
    ExpectingFills,
    ReadingFills,
}

// region: error boilerplate

#[derive(Debug)]
pub struct NodeStreamError(pub String);
impl std::error::Error for NodeStreamError {}
impl Display for NodeStreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "node iterator: {}", self.0)
    }
}

impl NodeStreamError {
    fn parser(e: JsonParseError) -> Self {
        Self(e.to_string())
    }

    fn msg(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl From<JsonParseError> for NodeStreamError {
    fn from(value: JsonParseError) -> Self {
        Self(format!("{value}"))
    }
}

// endregion: error boilerplate

impl<R: Read> From<R> for NodeStream<R> {
    fn from(value: R) -> Self {
        NodeStream {
            reader: ReaderJsonParser::new(value),
            stack: VecDeque::with_capacity(100),
            state: NodeStreamState::Default,
        }
    }
}

#[derive(Default)]
pub struct NodeDto {
    pub id: Option<String>,
    pub name: Option<String>,
    pub visible: Option<bool>,
    pub has_raster_fills: bool,
    pub hasher: xxhash_rust::xxh64::Xxh64,
}

macro_rules! parse_next {
    ($r:expr) => {
        match $r.parse_next() {
            Ok(value) => value,
            Err(e) => return Some(Err(NodeStreamError::parser(e))),
        }
    };
}

macro_rules! parse_next_value {
    ($r:expr, $t:path) => {
        match $r.parse_next() {
            Ok($t(value)) => Some(value),
            Ok(JsonEvent::Null) => None,
            Ok(_) => return Some(Err(NodeStreamError::msg("unexpected token"))),
            Err(e) => return Some(Err(NodeStreamError::parser(e))),
        }
    };
}

impl<R: Read> Iterator for NodeStream<R> {
    type Item = std::result::Result<Node, NodeStreamError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let event = parse_next!(self.reader);

            if let Some(dto) = self.stack.back_mut() {
                update_hash(dto, &event);
            }

            use NodeStreamState::*;
            match self.state {
                Default => match event {
                    JsonEvent::StartObject => self.stack.push_back(NodeDto::default()),
                    JsonEvent::EndObject => {
                        let Some(dto) = self.stack.pop_back() else {
                            continue;
                        };

                        if let Some(parent) = self.stack.back_mut() {
                            parent.hasher.write_u64(dto.hasher.digest());
                        }

                        if let NodeDto {
                            id: Some(id),
                            name: Some(name),
                            visible,
                            has_raster_fills,
                            hasher,
                        } = dto
                        {
                            return Some(Ok(Node {
                                id,
                                name,
                                visible: visible.unwrap_or(true),
                                has_raster_fills,
                                hash: hasher.digest(),
                            }));
                        }
                    }
                    JsonEvent::ObjectKey(key) => match key.as_ref() {
                        "id" => {
                            let id = parse_next_value!(self.reader, JsonEvent::String);
                            if let (Some(dto), Some(id)) = (self.stack.back_mut(), id) {
                                dto.id = Some(id.to_string());
                                update_hash(dto, &JsonEvent::String(id));
                            }
                        }
                        "name" => {
                            let name = parse_next_value!(self.reader, JsonEvent::String);
                            if let (Some(dto), Some(name)) = (self.stack.back_mut(), name) {
                                dto.name = Some(name.to_string());
                                update_hash(dto, &JsonEvent::String(name));
                            }
                        }
                        "visible" => {
                            let visible = parse_next_value!(self.reader, JsonEvent::Boolean);
                            if let (Some(dto), Some(visible)) = (self.stack.back_mut(), visible) {
                                dto.visible = Some(visible);
                                update_hash(dto, &JsonEvent::Boolean(visible));
                            }
                        }
                        "fills" => self.state = ExpectingFills,
                        _ => (), // just ignore
                    },
                    JsonEvent::Eof => return None,
                    _ => (), // just ignore
                },
                ExpectingFills => match event {
                    JsonEvent::StartArray => self.state = ReadingFills,
                    _ => return Some(Err(NodeStreamError::msg("reading fills: expected array"))),
                },
                ReadingFills => match event {
                    JsonEvent::EndArray => self.state = Default,
                    JsonEvent::ObjectKey(key) => match key.as_ref() {
                        "type" => {
                            let fill_type = parse_next_value!(self.reader, JsonEvent::String);
                            if let (Some(dto), Some(fill_type)) = (self.stack.back_mut(), fill_type)
                            {
                                dto.has_raster_fills = fill_type == "IMAGE";
                                update_hash(dto, &JsonEvent::String(fill_type));
                            }
                        }
                        _ => (), // irrelevant
                    },
                    _ => (),
                },
            }
        }
    }
}

fn update_hash(dto: &mut NodeDto, event: &JsonEvent<'_>) {
    match &event {
        JsonEvent::StartObject => dto.hasher.write_u8(b'{'),
        JsonEvent::EndObject => dto.hasher.write_u8(b'}'),
        JsonEvent::StartArray => dto.hasher.write_u8(b'['),
        JsonEvent::EndArray => dto.hasher.write_u8(b']'),
        JsonEvent::Number(s) => dto.hasher.write(s.as_bytes()),
        JsonEvent::Boolean(b) => dto.hasher.write_u8(if *b { 1 } else { 0 }),
        JsonEvent::Null => dto.hasher.write(b"null"),
        JsonEvent::String(s) => dto.hasher.write(s.as_bytes()),
        JsonEvent::ObjectKey(key) => {
            dto.hasher.write(b"key:");
            dto.hasher.write(key.as_bytes());
        }
        JsonEvent::Eof => (),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::BufReader;

    #[test]
    fn parse_single_relevant_node() {
        // Given
        let json = r#" {"id":"0-1","name":"Icon / Coffee"} "#;
        let expected_nodes: &[_] = &[Node {
            id: "0-1".to_string(),
            name: "Icon / Coffee".to_string(),
            visible: true,
            has_raster_fills: false,
            hash: 17989534996362531265,
        }];

        // When
        let iter = NodeStream::from(BufReader::new(json.as_bytes()));
        let actual_nodes = iter.collect::<std::result::Result<Vec<Node>, _>>().unwrap();

        // Then
        assert_eq!(expected_nodes, actual_nodes);
    }

    #[test]
    fn parse_multiple_relevant_nodes_inside_multiple_irrelevant() {
        // Given
        let json = r#"
        {
            "id":"0-1",
            "children": [
                {
                    "id":"0-2",
                    "children": [
                        {
                            "id":"0-3",
                            "name":"Icon / Leaf"
                        }
                    ]
                },
                {
                    "id":"0-4",
                    "name":"Icon / Coffee"
                }
            ]
        }
        "#;
        let expected_nodes: &[_] = &[
            Node {
                id: "0-3".to_string(),
                name: "Icon / Leaf".to_string(),
                visible: true,
                has_raster_fills: false,
                hash: 15384143807047250368,
            },
            Node {
                id: "0-4".to_string(),
                name: "Icon / Coffee".to_string(),
                visible: true,
                has_raster_fills: false,
                hash: 9053959461058680569,
            },
        ];

        // When
        let iter = NodeStream::from(BufReader::new(json.as_bytes()));
        let actual_nodes = iter.collect::<std::result::Result<Vec<Node>, _>>().unwrap();

        // Then
        assert_eq!(expected_nodes, actual_nodes);
    }

    #[test]
    fn parse_single_relevant_node_with_raster_fill() {
        // Given
        let json = r#"
        {
            "id":"0-1",
            "name":"Icon / Coffee",
            "fills": [ {"blendMode":"NORMAL","type":"IMAGE"} ]
        } "#;
        let expected_nodes: &[_] = &[Node {
            id: "0-1".to_string(),
            name: "Icon / Coffee".to_string(),
            visible: true,
            has_raster_fills: true,
            hash: 10040623593275229939,
        }];

        // When
        let iter = NodeStream::from(BufReader::new(json.as_bytes()));
        let actual_nodes = iter.collect::<std::result::Result<Vec<Node>, _>>().unwrap();

        // Then
        assert_eq!(expected_nodes, actual_nodes);
    }

    #[test]
    fn parse_multiple_relevant_nodes_with_raster_fills_inside_multiple_irrelevant() {
        // Given
        let json = r#"
        {
            "id":"0-1",
            "children": [
                {
                    "id":"0-2",
                    "children": [
                        {
                            "id":"0-3",
                            "name":"Icon / Leaf",
                            "fills": [ {"blendMode":"MULTIPLY","type":"IMAGE"} ]
                        }
                    ]
                },
                {
                    "id":"0-4",
                    "name":"Icon / Coffee",
                    "fills": [ {"blendMode":"NORMAL","type":"IMAGE"} ]
                }
            ]
        }
        "#;
        let expected_nodes: &[_] = &[
            Node {
                id: "0-3".to_string(),
                name: "Icon / Leaf".to_string(),
                visible: true,
                has_raster_fills: true,
                hash: 15472301963993704598,
            },
            Node {
                id: "0-4".to_string(),
                name: "Icon / Coffee".to_string(),
                visible: true,
                has_raster_fills: true,
                hash: 16736626037856730157,
            },
        ];

        // When
        let iter = NodeStream::from(BufReader::new(json.as_bytes()));
        let actual_nodes = iter.collect::<std::result::Result<Vec<Node>, _>>().unwrap();

        // Then
        assert_eq!(expected_nodes, actual_nodes);
    }

    #[test]
    fn similar_nodes_has_different_hash() {
        // Given
        let json = r#"
        {
            "id":"0-1",
            "children": [
                {
                    "id":"0-2",
                    "children": [
                        {
                            "id":"0-3",
                            "name":"Icon / Coffee"
                        }
                    ]
                },
                {
                    "id":"0-4",
                    "name":"Icon / Coffee"
                }
            ]
        }
        "#;

        // When
        let iter = NodeStream::from(BufReader::new(json.as_bytes()));
        let actual_nodes = iter.collect::<std::result::Result<Vec<Node>, _>>().unwrap();
        let node1 = actual_nodes.first().unwrap();
        let node2 = actual_nodes.last().unwrap();

        // Then
        assert_ne!(node1.hash, node2.hash);
    }

    #[test]
    fn fills_content_affects_hash() {
        // Given
        let json = r#"
        {
            "id":"0-1",
            "children": [
                {
                    "id":"0-2",
                    "children": [
                        {
                            "id":"0-3",
                            "name":"Icon / Coffee",
                            "fills": [ {"blendMode":"MULTIPLY","type":"IMAGE"} ]
                        }
                    ]
                },
                {
                    "id":"0-3",
                    "name":"Icon / Coffee",
                    "fills": [ {"blendMode":"MULTIPLY","type":"GRADIENT"} ]
                }
            ]
        }
        "#;

        // When
        let iter = NodeStream::from(BufReader::new(json.as_bytes()));
        let actual_nodes = iter.collect::<std::result::Result<Vec<Node>, _>>().unwrap();
        let node1 = actual_nodes.first().unwrap();
        let node2 = actual_nodes.last().unwrap();

        // Then
        assert_ne!(node1.hash, node2.hash);
    }
}
