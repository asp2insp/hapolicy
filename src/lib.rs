 // HAPolicy is a hierarchical token system with a language similar to IAM policies.
// Its goal is to allow cross-service authentication grants for specific resources.
// It aims to be a more opinionated and structured version of JSON Web Tokens (JWT).
//
// An example HAPolicy:
//
// {
//   "Version": "2015-10-7",
//   "Statements": [
//     {
//       "Effect": "Allow",
//       "Actions": [
//         "myservice:MyAction1",
//         "myservice:MyAction2"
//       ],
//       "Resources": [
//         "ht:myapp:myservice:hierarchical/path/*"
//       ]
//     }
//   ],
//   "Signature": ""
// }

// type Statement struct {
// 	Effect    string
// 	Actions   []string
// 	Resources []string
// }
//
// type UnsignedHToken struct {
// 	Version    string
// 	Statements []Statement
// 	Expiration uint64
// }
//
// type HToken struct {
// 	UnsignedHToken
// 	Signature string
// }
//
// type Resource struct {
// 	ServicePath []string
// 	Hierarchy   []string
// }

mod glob;

#[test]
fn it_works() {
}
