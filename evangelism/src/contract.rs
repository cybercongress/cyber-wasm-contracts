use cosmwasm::errors::{contract_err, Result, unauthorized};
use cosmwasm::traits::{Api, Extern, Storage};
use cosmwasm::types::{Params, Response};
use cw_storage::serialize;

use crate::msg::{HandleMsg, InitMsg, QueryMsg, ResolveEvangelistResponse};
use crate::state::{config, config_read, EvangelistRecord, resolver, resolver_read, State};

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
        HandleMsg::Believe { nickname, keybase, github} => try_believe(deps, params, nickname, keybase, github),
        HandleMsg::Bless { nickname } => try_bless(deps, params, nickname),
        HandleMsg::Unbless { nickname } => try_unbless(deps, params, nickname),
    }
}

pub fn try_believe<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    params: Params,
    nickname: String,
    keybase: String,
    github: String,
) -> Result<Response> {
    let key = nickname.as_bytes();
    let record = EvangelistRecord {
        cyber: params.message.signer,
        nickname: nickname.clone(),
        keybase,
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
    })?;

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
    })?;

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
        keybase: record.keybase,
        github: record.github,
        accepted: record.accepted,
    };

    serialize(&resp)
}
