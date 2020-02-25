use cosmwasm::mock::{mock_params, MockApi, MockStorage};
use cosmwasm::traits::Api;
use cosmwasm::types::HumanAddr;
use cosmwasm_vm::Instance;
use cosmwasm_vm::testing::{handle, init, mock_instance, query};
use cw_storage::deserialize;

use evangelism::msg::{HandleMsg, InitMsg, QueryMsg, ResolveEvangelistResponse};
use evangelism::state::State;

/**
This integration test tries to run and call the generated wasm.
It depends on a release build being available already. You can create that with:

cargo wasm && wasm-gc ./target/wasm32-unknown-unknown/release/hackatom.wasm

Then running `cargo test` will validate we can properly call into that generated data.

You can easily convert unit tests to integration tests.
1. First copy them over verbatum,
2. Then change
    let mut deps = dependencies(20);
To
    let mut deps = mock_instance(WASM);
3. If you access raw storage, where ever you see something like:
    deps.storage.get(CONFIG_KEY).expect("no data stored");
 replace it with:
    deps.with_storage(|store| {
        let data = store.get(CONFIG_KEY).expect("no data stored");
        //...
    });
4. Anywhere you see query(&deps, ...) you must replace it with query(&mut deps, ...)
5. When matching on error codes, you can not use Error types, but rather must use strings:
     match res {
         Err(Error::Unauthorized{..}) => {},
         _ => panic!("Must return unauthorized error"),
     }
     becomes:
     match res {
        ContractResult::Err(msg) => assert_eq!(msg, "Unauthorized"),
        _ => panic!("Expected error"),
     }



**/

// This line will test the output of cargo wasm
static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/evangelism.wasm");
// You can uncomment this line instead to test productionified build from cosmwasm-opt
// static WASM: &[u8] = include_bytes!("../contract.wasm");

fn assert_evangelist_record(mut deps: &mut Instance<MockStorage, MockApi>, cyber: &str, nickname: &str, accepted: bool) {
    let res = query(
        &mut deps,
        QueryMsg::ResolveEvangelist {
            nickname: nickname.to_string(),
        },
    )
        .unwrap();

    let value: ResolveEvangelistResponse = deserialize(&res).unwrap();
    assert_eq!(HumanAddr::from(cyber), value.cyber);
    assert_eq!(nickname, value.nickname);
    assert_eq!(accepted, value.accepted);
}

fn assert_config_state(mut deps: &mut Instance<MockStorage, MockApi>, expected: State) {
    let res = query(&mut deps, QueryMsg::Config {}).unwrap();
    let value: State = deserialize(&res).unwrap();
    assert_eq!(value, expected);
}

fn mock_init(
    mut deps: &mut Instance<MockStorage, MockApi>,
) {
    let msg = InitMsg { };
    let params = mock_params(&deps.api, "creator", &[], &[]);
    let _res = init(&mut deps, params, msg).unwrap();
}

fn mock_evangelist_believe(
    mut deps: &mut Instance<MockStorage, MockApi>,
) {
    let params = mock_params(&deps.api, "alice", &[], &[]);
    let msg = HandleMsg::Believe {
        nickname: "alice_nickname".to_string(),
        keybase: "alice_keybase".to_string(),
        github:   "alice_github".to_string(),
    };
    let _res =
        handle(&mut deps, params, msg).unwrap();
}

fn mock_creator_bless(
    mut deps: &mut Instance<MockStorage, MockApi>,
) {
    let params = mock_params(&deps.api, "creator", &[], &[]);
    let msg = HandleMsg::Bless {
        nickname: "alice_nickname".to_string(),
    };
    let _res =
        handle(&mut deps, params, msg).unwrap();
}

fn mock_creator_unbless(
    mut deps: &mut Instance<MockStorage, MockApi>,
) {
    let params = mock_params(&deps.api, "creator", &[], &[]);
    let msg = HandleMsg::Unbless {
        nickname: "alice_nickname".to_string(),
    };
    let _res =
        handle(&mut deps, params, msg).unwrap();
}

#[test]
fn proper_init() {
    let mut deps = mock_instance(WASM);
    mock_init(&mut deps);
    let expected_owner = deps.api.canonical_address(&HumanAddr("creator".to_string())).unwrap();

    assert_config_state(
        &mut deps,
        State {
            owner:  expected_owner
        }
    )
}

#[test]
fn proper_evangelist_believe() {
    let mut deps = mock_instance(WASM);
    mock_init(&mut deps);
    mock_evangelist_believe(&mut deps);

    assert_evangelist_record(&mut deps, "alice", "alice_nickname", false);
}

#[test]
fn proper_creator_bless() {
    let mut deps = mock_instance(WASM);
    mock_init(&mut deps);
    mock_evangelist_believe(&mut deps);
    mock_creator_bless(&mut deps);

    assert_evangelist_record(&mut deps, "alice", "alice_nickname", true);
}

#[test]
fn proper_creator_unbless() {
    let mut deps = mock_instance(WASM);
    mock_init(&mut deps);
    mock_evangelist_believe(&mut deps);
    mock_creator_bless(&mut deps);
    mock_creator_unbless(&mut deps);

    assert_evangelist_record(&mut deps, "alice", "alice_nickname", false);
}
