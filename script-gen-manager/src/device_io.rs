use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SimulatedDeviceIO {
    query_response_map: HashMap<String, String>,
}

impl SimulatedDeviceIO {
    pub fn new() -> Self {
        let mut query_response_map = HashMap::new();

        // based on the query response map from the SimulatedDeviceIO.java in TSP Express
        query_response_map.insert(String::from("SEARCH"), String::from("localnode.smua,localnode.smub,node[37].smua,node[37].smub,node[41].smua,node[41].smub,node[45].smua,node[45].smub"));
        query_response_map.insert(String::from("IDENTIFY_node[37]"), String::from("2602A,2.1.1,6700037A,Simulated Device,60.0"));
        query_response_map.insert(String::from("IDENTIFY_node[41]"), String::from("2612A,2.1.1,6700041A,Simulated Device,60.0"));
        query_response_map.insert(String::from("IDENTIFY_node[45]"), String::from("2636A,2.1.1,6700045A,Simulated Device,60.0"));
        query_response_map.insert(String::from("IDENTIFY"), String::from("2602A,2.1.1,6700001A,Simulated Device,60.0"));

        SimulatedDeviceIO { query_response_map }
    }

    pub fn get_query_response(&self, key: String) -> String {
        match self.query_response_map.get(&key) {
            Some(value) => value.to_string(),
            None => "".to_string(),
        }
    }
}

//struct SocketDeviceIO;
