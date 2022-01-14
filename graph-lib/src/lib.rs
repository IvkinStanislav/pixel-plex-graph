use std::{
    io::{BufRead, Write, BufWriter},
    collections::{HashMap, HashSet, VecDeque},
    rc::Rc,
};
use errors::GraphError;

mod utils;
mod errors;

pub type DefaultGraphIdType = u32;

#[derive(Debug)]
pub struct Graph<VT, ET> {
    vertices: HashMap<DefaultGraphIdType, Vertex<VT, ET>>,
    r#type: GraphType,
}

#[derive(Debug)]
pub enum GraphType {
    Directed,
    Undirected,
}

#[derive(Debug)]
pub struct Vertex<VT, ET> {
    id: DefaultGraphIdType,
    value: Option<VT>,
    edge_directions: Vec<EdgeDirection<ET>>
}

#[derive(Debug)]
pub struct EdgeDirection<ET> {
    to_vertex_id: DefaultGraphIdType,
    value: Rc<Option<ET>>,
    r#type: EdgeDirectionType,
}

/// EdgeDirectionType.Strong - простое ребро
/// EdgeDirectionType.Weak  - зеркальная копия ребра, создаваемая в неориентированно графе для настоящего ребра (детали реализации)
#[derive(Debug)]
enum EdgeDirectionType {
    Strong,
    Weak,
}

impl<VT, ET> Vertex<VT, ET> {
    pub fn new(id: DefaultGraphIdType, value: Option<VT>) -> Vertex<VT, ET> {
        Vertex {
            id,
            value,
            edge_directions: Vec::new()
        }
    }
}

impl<ET> EdgeDirection<ET> {
    pub fn new(to_vertex_id: DefaultGraphIdType, value: Rc<Option<ET>>) -> EdgeDirection<ET> {
        EdgeDirection {
            to_vertex_id,
            value,
            r#type: EdgeDirectionType::Strong
        }
    }

    pub fn new_weak(to_vertex_id: DefaultGraphIdType, value: Rc<Option<ET>>) -> EdgeDirection<ET> {
        EdgeDirection {
            to_vertex_id,
            value,
            r#type: EdgeDirectionType::Weak
        }
    }
}

impl<ET> PartialEq for EdgeDirection<ET>  {
    fn eq(&self, other: &Self) -> bool {
        self.to_vertex_id == other.to_vertex_id
    }
}
impl<ET> Eq for EdgeDirection<ET> {}

impl<VT, ET> Graph<VT, ET> {
    pub fn new(r#type: GraphType) -> Graph<VT, ET> {
        Graph {
            vertices: HashMap::new(),
            r#type
        }
    }

    pub fn add_vertex(&mut self, vertex: Vertex<VT, ET>) -> Result<(), GraphError> {
        if self.vertices.contains_key(&vertex.id) {
            return Err(GraphError::VertexAlreadyExist(vertex.id));
        }
        self.vertices.insert(vertex.id, vertex);
        Ok(())
    }

    pub fn delete_vertex(&mut self, vertex_id: DefaultGraphIdType)  {
        self.vertices.remove(&vertex_id);
        for vertex in self.vertices.values_mut() {
            utils::remove_from_vec(&mut vertex.edge_directions, |edge_direction| edge_direction.to_vertex_id == vertex_id);
        }
    }

    pub fn add_edge(&mut self, from_id: DefaultGraphIdType, to_id: DefaultGraphIdType, value: Option<ET>) -> Result<(), GraphError> {
        let value = Rc::new(value);
        match self.r#type {
            GraphType::Undirected => {
                self.add_edge_direction(from_id, to_id, Rc::clone(&value), EdgeDirectionType::Strong)?;
                self.add_edge_direction(to_id, from_id, value, EdgeDirectionType::Weak)?;
                Ok(())
            }
            GraphType::Directed => {
                if !self.vertices.contains_key(&to_id) {
                    return Err(GraphError::VertexNotFound(to_id));
                }
                self.add_edge_direction(from_id, to_id, value, EdgeDirectionType::Strong)?;
                Ok(())
            }
        }
    }

    pub fn delete_edge(&mut self, from_id: DefaultGraphIdType, to_id: DefaultGraphIdType) {
        match self.r#type {
            GraphType::Undirected => {
                self.delete_edge_direction(from_id, to_id);
                self.delete_edge_direction(to_id, from_id);
            }
            GraphType::Directed => {
                self.delete_edge_direction(from_id, to_id);
            }
        }
    }

    pub fn bfs_random_start(&self) -> Result<Vec<(DefaultGraphIdType, Option<&VT>, Vec<DefaultGraphIdType>)>, GraphError> {
        let vertex_id = self.vertices.keys().next();
        if let Some(vertex_id) = vertex_id {
            self.bfs(*vertex_id)
        }
        else {
            Ok(vec![])
        }
    }

    /// Список из идентификатора вершины, соседних идентификаторов вершин и значения вершины
    pub fn bfs(&self, start_id: DefaultGraphIdType) -> Result<Vec<(DefaultGraphIdType, Option<&VT>, Vec<DefaultGraphIdType>)>, GraphError> {
        let start_vertex = self.vertices.get(&start_id)
            .ok_or(GraphError::VertexNotFound(start_id))?;

        let mut result = Vec::new();
        let mut queue_vertex = VecDeque::new();
        let mut visited_vertices = HashSet::new();
        queue_vertex.push_back(start_vertex);

        while !queue_vertex.is_empty() {
            if let Some(current_vertex) = queue_vertex.pop_front() {
                if visited_vertices.contains(&current_vertex.id) {
                    continue;
                };
                visited_vertices.insert(current_vertex.id);
                let neighbours: Vec<_> = current_vertex.edge_directions
                    .iter()
                    .filter_map(|edge_direction| self.vertices.get(&edge_direction.to_vertex_id))
                    .collect();
                let neighbour_ids = neighbours
                    .iter()
                    .map(|vertex| vertex.id)
                    .collect();
                neighbours
                    .iter()
                    .filter(|&vertex| !visited_vertices.contains(&vertex.id))
                    .for_each(|vertex| queue_vertex.push_back(vertex));

                result.push((current_vertex.id, current_vertex.value.as_ref(), neighbour_ids));
            }
            else {
                break;
            }
        }

        Ok(result)
    }

    fn add_edge_direction(
        &mut self,
        from_id: DefaultGraphIdType,
        to_id: DefaultGraphIdType,
        value: Rc<Option<ET>>,
        edge_direction_type: EdgeDirectionType
    ) -> Result<(), GraphError> {
        let vertex_from = self.vertices.get_mut(&from_id)
            .ok_or(GraphError::VertexNotFound(from_id))?;

        let edge_to = match edge_direction_type {
            EdgeDirectionType::Strong => EdgeDirection::new(to_id, value),
            EdgeDirectionType::Weak => EdgeDirection::new_weak(to_id, value),
        };
        if vertex_from.edge_directions.contains(&edge_to) {
            return Ok(());
        }
        vertex_from.edge_directions.push(edge_to);

        Ok(())
    }
    
    fn delete_edge_direction(&mut self, from_id: DefaultGraphIdType, to_id: DefaultGraphIdType) {
        let vertex_from = self.vertices.get_mut(&from_id);
        if let Some(vertex_from) = vertex_from {
            utils::remove_from_vec(&mut vertex_from.edge_directions, |edge_direction| edge_direction.to_vertex_id == to_id);
        }
    }

    fn contains_vertex(&self, vertex_id: DefaultGraphIdType) -> bool {
        self.vertices.contains_key(&vertex_id)
    }
}

#[derive(Debug)]
enum ScanState {
    Vertex,
    Edge,
}

const VERTEX_EDGE_DELEMITER: &str = "#";
const DATA_DELIMITER: &str = " ";

impl Graph<String, String> {
    pub fn serialize<W: Write>(&self, buf_writer: &mut BufWriter<W>) -> Result<(), GraphError> {
        for vertex in self.vertices.values() {
            if let Some(vertex_value) = &vertex.value {
                write!(buf_writer, "{} {}\n", vertex.id, vertex_value)?;
            } else {
                write!(buf_writer, "{}\n", vertex.id)?;
            }
        }

        write!(buf_writer, "{}\n", VERTEX_EDGE_DELEMITER)?;

        for vertex in self.vertices.values() {
            for edge_direction in &vertex.edge_directions {
                if let EdgeDirectionType::Weak = edge_direction.r#type {
                    continue;
                }
                let (to_id, from_id) = (vertex.id, edge_direction.to_vertex_id);
                if let Some(edge_value) = &edge_direction.value.as_ref() {
                    write!(buf_writer, "{} {} {}\n", to_id, from_id, edge_value)?;
                } else {
                    write!(buf_writer, "{} {}\n", to_id, from_id)?;
                }
            }
        }

        Ok(())
    }

    pub fn deserialize<BR: BufRead>(reader: BR) -> Result<Graph<String, String>, GraphError> {
        let mut graph = Graph::new(GraphType::Undirected);
        let mut scan_state = ScanState::Vertex;

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            match scan_state {
                ScanState::Vertex => {
                    let vertex = Graph::parse_vertex(&line);
                    match vertex {
                        Ok(vertex) => {
                            graph.add_vertex(vertex)?;
                        },
                        Err(error) => {
                            if Graph::is_delimiter(line) {
                                scan_state = ScanState::Edge;
                                continue;
                            }
                            else {
                                return Err(error);
                            }
                        }
                    }
                },
                ScanState::Edge => {
                    let (to, from, value) = Graph::parse_edge(&line, &graph)?;
                    graph.add_edge(to, from, value)?;
                }
            }
        };

        Ok(graph)
    }

    fn parse_vertex(line: &str) -> Result<Vertex<String, String>, GraphError> {
        let mut vertex_data = line.split(DATA_DELIMITER);
    
        let vertex_id = vertex_data.next()
            .ok_or(GraphError::ParseVertexId(line.to_owned()))?
            .parse::<u32>()
            .map_err(|_| GraphError::WrongVertexIdType(line.to_owned()))?;
        let vertex_value: String = vertex_data.collect::<Vec<&str>>().join(DATA_DELIMITER);
        let vertex_value = if vertex_value.is_empty() {
            None
        }
        else {
            Some(vertex_value)
        };
    
        Ok(Vertex::new(vertex_id, vertex_value))
    }
    
    fn is_delimiter(line: &str) -> bool {
        line == VERTEX_EDGE_DELEMITER
    }
    
    /// Возвращает кортеж из двух инцидентных вершин и значения ребра
    fn parse_edge(line: &str, graph: &Graph<String, String>) -> Result<(DefaultGraphIdType, DefaultGraphIdType, Option<String>), GraphError> {
        let mut edge_data = line.split(DATA_DELIMITER);
    
        let first_vertex_id = edge_data.next()
            .ok_or(GraphError::ParseVertexId(line.to_owned()))?
            .parse::<DefaultGraphIdType>()
            .map_err(|_| GraphError::WrongVertexIdType(line.to_owned()))?;
        let second_vertex_id = edge_data.next()
            .ok_or(GraphError::ParseVertexId(line.to_owned()))?
            .parse::<DefaultGraphIdType>()
            .map_err(|_| GraphError::WrongVertexIdType(line.to_owned()))?;
        let edge_value: String = edge_data.collect::<Vec<&str>>().join(DATA_DELIMITER);
        let edge_value = if edge_value.is_empty() {
            None
        }
        else {
            Some(edge_value)
        };
    
        if !graph.contains_vertex(first_vertex_id) {
            return Err(GraphError::VertexNotFound(first_vertex_id));
        };
        if !graph.contains_vertex(second_vertex_id) {
            return Err(GraphError::VertexNotFound(second_vertex_id));
        };
        Ok((
            first_vertex_id,
            second_vertex_id,
            edge_value
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{BufReader, BufWriter},
        collections::HashSet,
    };
    use anyhow::{
        Result,
        bail,
    };

    const TGF_GRAPH: &str = "1 January
2 March
3 April
4 May
5 December
6 June
7 September
#
1 2
3 2
4 3
5 1 Happy New Year!
5 3 April Fools Day
6 3
6 1
7 5
7 6
7 1";

    #[test]
    fn deserialize_serialize() -> Result<()> {
        let reader = BufReader::new(TGF_GRAPH.as_bytes());
        let graph = Graph::deserialize(reader)?;
        
        let mut bufer = BufWriter::new(Vec::new());
        graph.serialize(&mut bufer)?;
        let serialized_graph = String::from_utf8(bufer.into_inner()?)?;

        let mut original_lines = HashSet::new();
        let mut serialized_lines = HashSet::new();
        original_lines.extend(TGF_GRAPH.lines());
        serialized_lines.extend(serialized_graph.lines());

        if original_lines == serialized_lines {
            Ok(())
        } else {
            bail!("serialized graph not equals original graph")
        }
    }

    #[test]
    fn unique_vertex_id() -> Result<()> {
        let mut graph = Graph::<(), ()>::new(GraphType::Undirected);
        graph.add_vertex(Vertex::new(1, None))?;
        let adding_result = graph.add_vertex(Vertex::new(1, None));

        if let Err(GraphError::VertexAlreadyExist(_)) = adding_result {
            Ok(())
        } else {
            bail!("serialized graph not equals original graph")
        }
    }

    #[test]
    fn bfs_undirected() -> Result<()> {
        const VERTEX_ID_1: DefaultGraphIdType = 1;
        const VERTEX_ID_2: DefaultGraphIdType = 2;
        let mut graph = Graph::<(), ()>::new(GraphType::Undirected);
        graph.add_vertex(Vertex::new(VERTEX_ID_1, None))?;
        graph.add_vertex(Vertex::new(VERTEX_ID_2, None))?;
        graph.add_edge(VERTEX_ID_1, VERTEX_ID_2, None)?;

        let vertex_ids: Vec<_>= graph.bfs(VERTEX_ID_1)?
            .iter()
            .map(|(id, _, _)| *id)
            .collect();
        if vertex_ids.contains(&VERTEX_ID_1) && vertex_ids.contains(&VERTEX_ID_2) {
            Ok(())
        } else {
            bail!("bfs return wrong result")
        }
    }

    #[test]
    fn bfs_directed() -> Result<()> {
        const VERTEX_ID_1: DefaultGraphIdType = 1;
        const VERTEX_ID_2: DefaultGraphIdType = 2;
        let mut graph = Graph::<(), ()>::new(GraphType::Directed);
        graph.add_vertex(Vertex::new(VERTEX_ID_1, None))?;
        graph.add_vertex(Vertex::new(VERTEX_ID_2, None))?;
        graph.add_edge(VERTEX_ID_1, VERTEX_ID_2, None)?;

        let vertex_ids: Vec<_>= graph.bfs(VERTEX_ID_1)?
            .iter()
            .map(|(id, _, _)| *id)
            .collect();
        if vertex_ids.contains(&VERTEX_ID_1) && vertex_ids.contains(&VERTEX_ID_2) {
            Ok(())
        } else {
            bail!("bfs return wrong result")
        }
    }

    #[test]
    fn bfs_directed_reverse() -> Result<()> {
        const VERTEX_ID_1: DefaultGraphIdType = 1;
        const VERTEX_ID_2: DefaultGraphIdType = 2;
        let mut graph = Graph::<(), ()>::new(GraphType::Directed);
        graph.add_vertex(Vertex::new(VERTEX_ID_1, None))?;
        graph.add_vertex(Vertex::new(VERTEX_ID_2, None))?;
        graph.add_edge(VERTEX_ID_2, VERTEX_ID_1, None)?;

        let vertex_ids: Vec<_>= graph.bfs(VERTEX_ID_1)?
            .iter()
            .map(|(id, _, _)| *id)
            .collect();
        if vertex_ids.contains(&VERTEX_ID_1) && !vertex_ids.contains(&VERTEX_ID_2) {
            Ok(())
        } else {
            bail!("bfs return wrong result")
        }
    }

    #[test]
    fn bfs_delete_edge() -> Result<()> {
        const VERTEX_ID_1: DefaultGraphIdType = 1;
        const VERTEX_ID_2: DefaultGraphIdType = 2;
        const VERTEX_ID_3: DefaultGraphIdType = 3;
        let mut graph = Graph::<(), ()>::new(GraphType::Undirected);
        graph.add_vertex(Vertex::new(VERTEX_ID_1, None))?;
        graph.add_vertex(Vertex::new(VERTEX_ID_2, None))?;
        graph.add_vertex(Vertex::new(VERTEX_ID_3, None))?;
        graph.add_edge(VERTEX_ID_1, VERTEX_ID_2, None)?;
        graph.add_edge(VERTEX_ID_2, VERTEX_ID_3, None)?;

        graph.delete_edge(VERTEX_ID_2, VERTEX_ID_3);

        let vertex_ids: Vec<_>= graph.bfs(VERTEX_ID_1)?
            .iter()
            .map(|(id, _, _)| *id)
            .collect();
        if vertex_ids.contains(&VERTEX_ID_1) && vertex_ids.contains(&VERTEX_ID_2) && !vertex_ids.contains(&VERTEX_ID_3) {
            Ok(())
        } else {
            bail!("bfs return wrong result")
        }
    }

    #[test]
    fn bfs_delete_vertex() -> Result<()> {
        const VERTEX_ID_1: DefaultGraphIdType = 1;
        const VERTEX_ID_2: DefaultGraphIdType = 2;
        const VERTEX_ID_3: DefaultGraphIdType = 3;
        let mut graph = Graph::<(), ()>::new(GraphType::Undirected);
        graph.add_vertex(Vertex::new(VERTEX_ID_1, None))?;
        graph.add_vertex(Vertex::new(VERTEX_ID_2, None))?;
        graph.add_vertex(Vertex::new(VERTEX_ID_3, None))?;
        graph.add_edge(VERTEX_ID_1, VERTEX_ID_2, None)?;
        graph.add_edge(VERTEX_ID_2, VERTEX_ID_3, None)?;

        graph.delete_vertex(VERTEX_ID_2);

        let vertex_ids: Vec<_>= graph.bfs(VERTEX_ID_1)?
            .iter()
            .map(|(id, _, _)| *id)
            .collect();
        if vertex_ids.contains(&VERTEX_ID_1) && !vertex_ids.contains(&VERTEX_ID_2) && !vertex_ids.contains(&VERTEX_ID_3) {
            Ok(())
        } else {
            bail!("bfs return wrong result")
        }
    }

    #[test]
    fn vertex_neighbours() -> Result<()> {
        const VERTEX_ID_1: DefaultGraphIdType = 1;
        const VERTEX_ID_2: DefaultGraphIdType = 2;
        const VERTEX_ID_3: DefaultGraphIdType = 3;
        let mut graph = Graph::<(), ()>::new(GraphType::Undirected);
        graph.add_vertex(Vertex::new(VERTEX_ID_1, None))?;
        graph.add_vertex(Vertex::new(VERTEX_ID_2, None))?;
        graph.add_vertex(Vertex::new(VERTEX_ID_3, None))?;
        graph.add_edge(VERTEX_ID_1, VERTEX_ID_2, None)?;
        graph.add_edge(VERTEX_ID_2, VERTEX_ID_3, None)?;

        let bfs_result = graph.bfs(VERTEX_ID_1)?;
        let vertex_id_2_neighbours = bfs_result
            .iter()
            .find(|(id, _, _)| VERTEX_ID_2 == *id)
            .map(|(_, _, edges)| edges)
            .unwrap();
        if vertex_id_2_neighbours.contains(&VERTEX_ID_1) && !vertex_id_2_neighbours.contains(&VERTEX_ID_2) && vertex_id_2_neighbours.contains(&VERTEX_ID_3) {
            Ok(())
        } else {
            bail!("bfs return wrong result")
        }
    }
}
