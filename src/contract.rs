#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};

use self::execute::update_config;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:howl-pack-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        owner: info.sender.clone(),
        dens_addr: msg.dens_addr.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("dens_addr", msg.dens_addr))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateNotifications {
            token_id,
            notifications,
        } => execute::update_notification(deps, info.sender, token_id, notifications),
        ExecuteMsg::UpdateConfig { owner, dens_addr } => {
            update_config(deps, info, owner, dens_addr)
        }
    }
}

pub mod execute {
    use cosmwasm_std::{Addr, QueryRequest, WasmQuery};
    use cw721::{Cw721QueryMsg, OwnerOfResponse};

    use crate::state::{Notification, CONFIG, NOTIFICATIONS};

    use super::*;

    pub fn update_notification(
        deps: DepsMut,
        sender: Addr,
        token_id: String,
        notifications: Vec<Notification>,
    ) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;

        let response: OwnerOfResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: config.dens_addr.to_string(),
                msg: to_binary(&Cw721QueryMsg::OwnerOf {
                    token_id: token_id.clone(),
                    include_expired: None,
                })?,
            }))?;

        if response.owner != sender {
            return Err(ContractError::Unauthorized {});
        }

        NOTIFICATIONS.save(deps.storage, token_id.as_str(), &notifications)?;

        Ok(Response::new()
            .add_attribute("action", "update_notification")
            .add_attribute("token_id", token_id)
            .add_attribute("notifications", format!("{:?}", &notifications)))
    }

    pub fn update_config(
        deps: DepsMut,
        info: MessageInfo,
        new_owner: Option<Addr>,
        new_dens_addr: Option<Addr>,
    ) -> Result<Response, ContractError> {
        let config = CONFIG.update(deps.storage, |mut config| -> Result<_, ContractError> {
            if info.sender != config.owner {
                return Err(ContractError::Unauthorized {});
            }

            if let Some(owner) = new_owner {
                config.owner = owner
            }

            if let Some(dens_addr) = new_dens_addr {
                config.dens_addr = dens_addr
            }

            Ok(config)
        })?;

        Ok(Response::new()
            .add_attribute("action", "update_config")
            .add_attribute("dens_addr", config.dens_addr)
            .add_attribute("owner", config.owner))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&query::config(deps)?),
        QueryMsg::GetNotifications { token_id } => {
            to_binary(&query::notifications(deps, token_id).unwrap_or_default())
        }
    }
}

pub mod query {
    use crate::state::{Config, Notification, CONFIG, NOTIFICATIONS};

    use super::*;

    pub fn config(deps: Deps) -> StdResult<Config> {
        let config = CONFIG.load(deps.storage)?;
        Ok(config)
    }

    pub fn notifications(deps: Deps, token_id: String) -> StdResult<Vec<Notification>> {
        let notifications = NOTIFICATIONS.load(deps.storage, token_id.as_str())?;
        Ok(notifications)
    }
}

#[cfg(test)]
mod tests {
    use crate::state::{EmailNotification, Notification};

    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{
        coins, from_binary, Addr, ContractResult, QuerierResult, SystemError, SystemResult,
        WasmQuery,
    };
    use cw721::{Cw721QueryMsg, OwnerOfResponse};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            dens_addr: Addr::unchecked("dens_addr"),
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetConfig {}).unwrap();
        let value: Config = from_binary(&res).unwrap();
        assert_eq!(String::from("dens_addr"), value.dens_addr.to_string());
    }

    #[test]
    fn update_config() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            dens_addr: Addr::unchecked("dens_addr"),
        };
        let auth_info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), auth_info.clone(), msg).unwrap();

        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::UpdateConfig {
            owner: Some(Addr::unchecked("anyone")),
            dens_addr: None,
        };

        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        let msg = ExecuteMsg::UpdateConfig {
            owner: Some(Addr::unchecked("new_owner")),
            dens_addr: Some(Addr::unchecked("new_dens_addr")),
        };
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetConfig {}).unwrap();
        let value: Config = from_binary(&res).unwrap();
        assert_eq!(String::from("new_dens_addr"), value.dens_addr.to_string());
        assert_eq!(String::from("new_owner"), value.owner.to_string());
    }

    #[test]
    fn update_notifications() {
        const DENS: &str = "dens_addr";
        const TOKEN_ID: &str = "token_id";
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            dens_addr: Addr::unchecked(DENS),
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        fn mock_resp() -> OwnerOfResponse {
            OwnerOfResponse {
                approvals: vec![],
                owner: String::from("owner"),
            }
        }

        deps.querier.update_wasm(|q| -> QuerierResult {
            if q == &(WasmQuery::Smart {
                contract_addr: DENS.to_string(),
                msg: to_binary(&Cw721QueryMsg::OwnerOf {
                    token_id: TOKEN_ID.to_string(),
                    include_expired: None,
                })
                .unwrap(),
            }) {
                SystemResult::Ok(ContractResult::Ok(to_binary(&mock_resp()).unwrap()))
            } else {
                SystemResult::Err(SystemError::NoSuchContract {
                    addr: DENS.to_string(),
                })
            }
        });

        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::UpdateNotifications {
            token_id: TOKEN_ID.to_string(),
            notifications: vec![],
        };
        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the owner of the dens can update notifications
        let auth_info = mock_info("owner", &coins(2, "token"));
        let msg = ExecuteMsg::UpdateNotifications {
            token_id: TOKEN_ID.to_string(),
            notifications: vec![Notification::Email(EmailNotification {
                encoded_addr: format!("encoded_addr"),
                masked_addr: format!("masked_addr"),
                preferences: format!("4"),
            })],
        };
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetNotifications {
                token_id: TOKEN_ID.to_string(),
            },
        )
        .unwrap();
        let value: Vec<Notification> = from_binary(&res).unwrap();
        assert_eq!(1, value.len());
        assert_eq!(
            Notification::Email(EmailNotification {
                encoded_addr: format!("encoded_addr"),
                masked_addr: format!("masked_addr"),
                preferences: format!("4"),
            }),
            value[0]
        );
    }
}
