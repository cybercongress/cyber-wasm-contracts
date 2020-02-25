use cosmwasm::errors::{Result, unauthorized, contract_err};
use cosmwasm::traits::{Api, Extern, Storage};
use cosmwasm::types::{CanonicalAddr, Params, Response};

use crate::msg::{HandleMsg, InitMsg, QueryMsg, ResolveEvangelistResponse};
use crate::state::{config, config_read, resolver, resolver_read, State, EvangelistRecord};

use cw_storage::{ serialize, deserialize };

pub fn init<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    params: Params,
    _msg: InitMsg,
) -> Result<Response> {
    let state = State {
        owner: params.message.signer,
    };

    config(&mut deps.storage).save(&state)?;

    Ok(Response::default())
}

pub fn handle<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    params: Params,
    msg: HandleMsg,
) -> Result<Response> {
    match msg {
        HandleMsg::Believe { nickname, telegram, github} => try_believe(deps, params, nickname, telegram, github),
        HandleMsg::Bless { nickname } => try_bless(deps, params, nickname),
        HandleMsg::Unbless { nickname } => try_unbless(deps, params, nickname),
    }
}

pub fn try_believe<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    params: Params,
    nickname: String,
    telegram: String,
    github: String,
) -> Result<Response> {
    let key = nickname.as_bytes();
    let record = EvangelistRecord {
        cyber: params.message.signer,
        nickname: nickname.clone(),
        telegram,
        github,
        accepted: false,
    };

    if let None = resolver(&mut deps.storage).may_load(key)? {
        resolver(&mut deps.storage).save(key, &record)?;
    } else {
        contract_err("Nickname is already taken")?;
    }

    Ok(Response::default())
}

pub fn try_bless<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    params: Params,
    nickname: String,
) -> Result<Response> {
    let config_state = config(&mut deps.storage).load()?;

    let key = nickname.as_bytes();

    resolver(&mut deps.storage).update(key, &|mut record| {
       if params.message.signer != config_state.owner {
           unauthorized()?;
       }

       record.accepted = true;
       Ok(record)
    });

    Ok(Response::default())
}

pub fn try_unbless<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    params: Params,
    nickname: String,
) -> Result<Response> {
    let config_state = config(&mut deps.storage).load()?;

    let key = nickname.as_bytes();

    resolver(&mut deps.storage).update(key, &|mut record| {
        if params.message.signer != config_state.owner {
            unauthorized()?;
        }

        record.accepted = false;
        Ok(record)
    });

    Ok(Response::default())
}

pub fn query<S: Storage, A: Api>(deps: &Extern<S, A>, msg: QueryMsg) -> Result<Vec<u8>> {
    match msg {
        QueryMsg::ResolveEvangelist { nickname } => query_resolver(deps, nickname),
        QueryMsg::Config {} => serialize(&config_read(&deps.storage).load()?),
    }
}

fn query_resolver<S: Storage, A: Api>(deps: &Extern<S, A>, nickname: String) -> Result<Vec<u8>> {
    let key = nickname.as_bytes();
    let record = resolver_read(&deps.storage).load(key)?;
    let address = deps.api.human_address(&record.cyber)?;

    let resp = ResolveEvangelistResponse {
        cyber: address,
        nickname: record.nickname,
        telegram: record.telegram,
        github: record.github,
        accepted: record.accepted,
    };

    serialize(&resp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm::mock::{dependencies, mock_params, MockStorage, MockApi};
    use cosmwasm::types::HumanAddr;
    use cosmwasm::traits::{Api};

    fn assert_evangelist_record(deps: &mut Extern<MockStorage, MockApi>, cyber: &str, nickname: &str, accepted: bool) {
        let res = query(
            &deps,
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

    fn assert_config_state(deps: &mut Extern<MockStorage, MockApi>, expected: State) {
        let res = query(&deps, QueryMsg::Config {}).unwrap();
        let value: State = deserialize(&res).unwrap();
        assert_eq!(value, expected);
    }

    fn mock_init(
        mut deps: &mut Extern<MockStorage, MockApi>,
    ) {
        let msg = InitMsg { };
        let params = mock_params(&deps.api, "creator", &[], &[]);
        let _res = init(&mut deps, params, msg).expect("contract successfully handles InitMsg");
    }

    fn mock_evangelist_believe(
        mut deps: &mut Extern<MockStorage, MockApi>,
    ) {
        let params = mock_params(&deps.api, "alice", &[], &[]);
        let msg = HandleMsg::Believe {
            nickname: "alice_nickname".to_string(),
            telegram: "alice_telegram".to_string(),
            github:   "alice_github".to_string(),
        };
        let _res =
            handle(&mut deps, params, msg).expect("contract successfully handles Believe message");
    }

    fn mock_creator_bless(
        mut deps: &mut Extern<MockStorage, MockApi>,
    ) {
        let params = mock_params(&deps.api, "creator", &[], &[]);
        let msg = HandleMsg::Bless {
            nickname: "alice_nickname".to_string(),
        };
        let _res =
            handle(&mut deps, params, msg).expect("contract successfully handles Bless message");
    }

    fn mock_creator_unbless(
        mut deps: &mut Extern<MockStorage, MockApi>,
    ) {
        let params = mock_params(&deps.api, "creator", &[], &[]);
        let msg = HandleMsg::Unbless {
            nickname: "alice_nickname".to_string(),
        };
        let _res =
            handle(&mut deps, params, msg).expect("contract successfully handles Unbless message");
    }

    #[test]
    fn proper_init() {
        let mut deps = dependencies(20);
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
        let mut deps = dependencies(20);
        mock_init(&mut deps);
        mock_evangelist_believe(&mut deps);

        assert_evangelist_record(&mut deps, "alice", "alice_nickname", false);
    }

    #[test]
    fn proper_creator_bless() {
        let mut deps = dependencies(20);
        mock_init(&mut deps);
        mock_evangelist_believe(&mut deps);
        mock_creator_bless(&mut deps);

        assert_evangelist_record(&mut deps, "alice", "alice_nickname", true);
    }

    fn proper_creator_unbless() {
        let mut deps = dependencies(20);
        mock_init(&mut deps);
        mock_evangelist_believe(&mut deps);
        mock_creator_bless(&mut deps);
        mock_creator_unbless(&mut deps);

        assert_evangelist_record(&mut deps, "alice", "alice_nickname", false);
    }
}
