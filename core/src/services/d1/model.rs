// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use crate::Error;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fmt::Debug;

/// response data from d1
#[derive(Deserialize, Debug)]
pub struct D1Response {
    pub result: Vec<D1Result>,
    pub success: bool,
    pub errors: Vec<D1Error>,
    pub messages: Vec<D1Message>,
}

impl D1Response {
    pub fn parse(bs: &Bytes) -> Result<D1Response, Error> {
        let response: D1Response = serde_json::from_slice(bs).map_err(|e| {
            Error::new(
                crate::ErrorKind::Unexpected,
                &format!("failed to parse error response: {}", e),
            )
        })?;

        if !response.success {
            return Err(Error::new(
                crate::ErrorKind::Unexpected,
                &String::from_utf8_lossy(bs),
            ));
        }
        Ok(response)
    }

    pub fn get_result(&self, key: &str) -> Option<Vec<u8>> {
        if self.result.is_empty() || self.result[0].results.is_empty() {
            return None;
        }
        let result = &self.result[0].results[0];
        let value = result.get(key);

        match value {
            Some(Value::Array(s)) => {
                let mut v = Vec::new();
                for i in s {
                    if let Value::Number(n) = i {
                        v.push(n.as_u64().unwrap() as u8);
                    }
                }
                Some(v)
            }
            _ => None,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct D1Result {
    pub meta: Meta,
    pub results: Vec<Map<String, Value>>,
    pub success: bool,
}

#[derive(Deserialize, Debug)]
pub struct Meta {
    pub served_by: String,
    pub duration: f64,
    pub changes: i32,
    pub last_row_id: i32,
    pub changed_db: bool,
    pub size_after: i32,
    pub rows_read: i32,
    pub rows_written: i32,
}

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct D1Error {
    pub message: String,
    pub code: i32,
}

#[derive(Deserialize, Debug)]
pub struct D1Message {
    pub message: String,
    pub code: i32,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_deserialize_get_object_json_response() {
        let data = r#"
        {
            "result": [
                {
                    "results": [
                        {
                            "CustomerId": "4",
                            "CompanyName": "Around the Horn",
                            "ContactName": "Thomas Hardy"
                        }
                    ],
                    "success": true,
                    "meta": {
                        "served_by": "v3-prod",
                        "duration": 0.2147,
                        "changes": 0,
                        "last_row_id": 0,
                        "changed_db": false,
                        "size_after": 2162688,
                        "rows_read": 3,
                        "rows_written": 2
                    }
                }
            ],
            "success": true,
            "errors": [],
            "messages": []
        }"#;
        let response: D1Response = serde_json::from_str(data).unwrap();
        println!("{:?}", response.result[0].results[0]);
    }
}
