use proxmox_api_macro::api;

use failure::Error;
use serde_json::{json, Value};

#[api(
    input: {
        properties: {
            value: {
                description: "The optional value with default.",
                optional: true,
                default: false,
            }
        }
    }
)]
/// Print the given message.
///
/// Returns: the input.
pub fn test_option(value: bool) -> Result<bool, Error> {
    Ok(value)
}

struct RpcEnv;
impl proxmox::api::RpcEnvironment for RpcEnv {
    fn set_result_attrib(&mut self, name: &str, value: Value) {
        let _ = (name, value);
        panic!("set_result_attrib called");
    }

    /// Query additional result data.
    fn get_result_attrib(&self, name: &str) -> Option<&Value> {
        let _ = name;
        panic!("get_result_attrib called");
    }

    /// The environment type
    fn env_type(&self) -> proxmox::api::RpcEnvironmentType {
        panic!("env_type called");
    }

    /// Set user name
    fn set_user(&mut self, user: Option<String>) {
        let _ = user;
        panic!("set_user called");
    }

    /// Get user name
    fn get_user(&self) -> Option<String> {
        panic!("get_user called");
    }
}

#[test]
fn test_invocations() {
    let mut env = RpcEnv;
    let value = api_function_test_option(json!({}), &API_METHOD_TEST_OPTION, &mut env)
        .expect("func with option should work");
    assert_eq!(value, false);

    let value = api_function_test_option(json!({"value": true}), &API_METHOD_TEST_OPTION, &mut env)
        .expect("func with option should work");
    assert_eq!(value, true);

    let value = api_function_test_option(json!({"value": false}), &API_METHOD_TEST_OPTION, &mut env)
        .expect("func with option should work");
    assert_eq!(value, false);
}
