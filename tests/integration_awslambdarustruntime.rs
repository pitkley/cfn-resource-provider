#![cfg(feature = "native-runtime")]
#![allow(dead_code)]

extern crate cfn_resource_provider as cfn;
extern crate lambda_runtime as lambda;
#[macro_use]
extern crate serde_derive;

use cfn::*;
use lambda::{error::HandlerError, lambda, Context};

#[derive(Debug, Clone, Deserialize)]
struct DummyType;

impl PhysicalResourceIdSuffixProvider for DummyType {
    fn physical_resource_id_suffix(&self) -> String {
        unimplemented!()
    }
}

fn lambda_handler(
    request: CfnRequest<DummyType>,
    context: Context,
) -> Result<Option<()>, HandlerError> {
    // Write some code that does something with the requests, specifically with its strongly typed
    // request properties.
    let result: Result<Option<()>, String> = Ok(None);

    cfn::process(request, &result).map_err(|e| context.new_error(&format!("{}", e)))?;
    Ok(None)
}

fn main() {
    lambda!(lambda_handler);
}
