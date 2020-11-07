var searchIndex = JSON.parse('{\
"cfn_resource_provider":{"doc":"cfn-resource-provider","i":[[3,"Ignored","cfn_resource_provider","This is a special struct that can be used in conjunction…",null,null],[4,"CfnRequest","","On stack modification, AWS CloudFormation sends out a…",null,null],[13,"Create","","Custom resource provider requests with `RequestType` set…",0,null],[12,"request_id","cfn_resource_provider::CfnRequest","A unique ID for the request.",1,null],[12,"response_url","","The response URL identifies a presigned S3 bucket that…",1,null],[12,"resource_type","","The template developer-chosen resource type of the custom…",1,null],[12,"logical_resource_id","","The template developer-chosen name (logical ID) of the…",1,null],[12,"stack_id","","The Amazon Resource Name (ARN) that identifies the stack…",1,null],[12,"resource_properties","","This field contains the contents of the `Properties`…",1,null],[13,"Delete","cfn_resource_provider","Custom resource provider requests with `RequestType` set…",0,null],[12,"request_id","cfn_resource_provider::CfnRequest","A unique ID for the request.",2,null],[12,"response_url","","The response URL identifies a presigned S3 bucket that…",2,null],[12,"resource_type","","The template developer-chosen resource type of the custom…",2,null],[12,"logical_resource_id","","The template developer-chosen name (logical ID) of the…",2,null],[12,"stack_id","","The Amazon Resource Name (ARN) that identifies the stack…",2,null],[12,"physical_resource_id","","A required custom resource provider-defined physical ID…",2,null],[12,"resource_properties","","This field contains the contents of the `Properties`…",2,null],[13,"Update","cfn_resource_provider","Custom resource provider requests with `RequestType` set…",0,null],[12,"request_id","cfn_resource_provider::CfnRequest","A unique ID for the request.",3,null],[12,"response_url","","The response URL identifies a presigned S3 bucket that…",3,null],[12,"resource_type","","The template developer-chosen resource type of the custom…",3,null],[12,"logical_resource_id","","The template developer-chosen name (logical ID) of the…",3,null],[12,"stack_id","","The Amazon Resource Name (ARN) that identifies the stack…",3,null],[12,"physical_resource_id","","A required custom resource provider-defined physical ID…",3,null],[12,"resource_properties","","This field contains the contents of the `Properties`…",3,null],[12,"old_resource_properties","","The resource property values that were previously declared…",3,null],[4,"CfnResponse","cfn_resource_provider","This enum represents the response expected by AWS…",null,null],[13,"Success","","Indicates that the modification of the custom resource…",4,null],[12,"request_id","cfn_resource_provider::CfnResponse","A unique ID for the request. This response value should be…",5,null],[12,"logical_resource_id","","The template developer-chosen name (logical ID) of the…",5,null],[12,"stack_id","","The Amazon Resource Name (ARN) that identifies the stack…",5,null],[12,"physical_resource_id","","This value should be an identifier unique to the custom…",5,null],[12,"no_echo","","Optional. Indicates whether to mask the output of the…",5,null],[12,"data","","Optional. The custom resource provider-defined name-value…",5,null],[13,"Failed","cfn_resource_provider","Indicates that the modification of the custom resource…",4,null],[12,"reason","cfn_resource_provider::CfnResponse","Describes the reason for a failure response.",6,null],[12,"request_id","","A unique ID for the request. This response value should be…",6,null],[12,"logical_resource_id","","The template developer-chosen name (logical ID) of the…",6,null],[12,"stack_id","","The Amazon Resource Name (ARN) that identifies the stack…",6,null],[12,"physical_resource_id","","This value should be an identifier unique to the custom…",6,null],[5,"process","cfn_resource_provider","Process an AWS CloudFormation custom resource request.",null,[[]]],[8,"PhysicalResourceIdSuffixProvider","","Every AWS CloudFormation resource, including custom…",null,null],[10,"physical_resource_id_suffix","","Creates a suffix that uniquely identifies the physical…",7,[[],["string",3]]],[11,"request_id","","The request ID field exists for all variants of the…",0,[[],["string",3]]],[11,"response_url","","The response URL field exists for all variants of the…",0,[[],["string",3]]],[11,"resource_type","","The resource type field exists for all variants of the…",0,[[],["string",3]]],[11,"logical_resource_id","","The logical resource ID field exists for all variants of…",0,[[],["string",3]]],[11,"stack_id","","The stack ID field exists for all variants of the…",0,[[],["string",3]]],[11,"physical_resource_id","","The physical resource ID field either exists or has to be…",0,[[],["string",3]]],[11,"resource_properties","","The resource properties field exists for all variants of…",0,[[]]],[11,"into_response","","This method turns a [`CfnRequest`] into a [`CfnResponse`],…",0,[[["result",4]],["cfnresponse",4]]],[11,"from","","",8,[[]]],[11,"into","","",8,[[]]],[11,"to_owned","","",8,[[]]],[11,"clone_into","","",8,[[]]],[11,"try_from","","",8,[[],["result",4]]],[11,"try_into","","",8,[[],["result",4]]],[11,"borrow","","",8,[[]]],[11,"borrow_mut","","",8,[[]]],[11,"type_id","","",8,[[],["typeid",3]]],[11,"try_into","","",8,[[],["result",4]]],[11,"from","","",0,[[]]],[11,"into","","",0,[[]]],[11,"to_owned","","",0,[[]]],[11,"clone_into","","",0,[[]]],[11,"try_from","","",0,[[],["result",4]]],[11,"try_into","","",0,[[],["result",4]]],[11,"borrow","","",0,[[]]],[11,"borrow_mut","","",0,[[]]],[11,"type_id","","",0,[[],["typeid",3]]],[11,"try_into","","",0,[[],["result",4]]],[11,"from","","",4,[[]]],[11,"into","","",4,[[]]],[11,"to_owned","","",4,[[]]],[11,"clone_into","","",4,[[]]],[11,"try_from","","",4,[[],["result",4]]],[11,"try_into","","",4,[[],["result",4]]],[11,"borrow","","",4,[[]]],[11,"borrow_mut","","",4,[[]]],[11,"type_id","","",4,[[],["typeid",3]]],[11,"try_into","","",4,[[],["result",4]]],[11,"physical_resource_id_suffix","","",8,[[],["string",3]]],[11,"clone","","",0,[[],["cfnrequest",4]]],[11,"clone","","",8,[[],["ignored",3]]],[11,"clone","","",4,[[],["cfnresponse",4]]],[11,"default","","",8,[[],["ignored",3]]],[11,"eq","","",0,[[["cfnrequest",4]]]],[11,"ne","","",0,[[["cfnrequest",4]]]],[11,"eq","","",8,[[["ignored",3]]]],[11,"eq","","",4,[[["cfnresponse",4]]]],[11,"ne","","",4,[[["cfnresponse",4]]]],[11,"fmt","","",0,[[["formatter",3]],["result",6]]],[11,"fmt","","",8,[[["formatter",3]],["result",6]]],[11,"fmt","","",4,[[["formatter",3]],["result",6]]],[11,"serialize","","",4,[[],["result",4]]],[11,"deserialize","","",0,[[],["result",4]]],[11,"deserialize","","",8,[[],[["ignored",3],["result",4]]]]],"p":[[4,"CfnRequest"],[13,"Create"],[13,"Delete"],[13,"Update"],[4,"CfnResponse"],[13,"Success"],[13,"Failed"],[8,"PhysicalResourceIdSuffixProvider"],[3,"Ignored"]]}\
}');
addSearchOptions(searchIndex);initSearch(searchIndex);