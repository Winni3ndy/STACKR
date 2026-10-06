use std::sync::Arc;

use crate::error::AppError;
use crate::operator::OperatorConfig;
use crate::AppState;

/// Per-request context that bundles the resolved operator config with shared app state.
/// All service functions that need operator-specific config accept `&RequestContext`.
#[derive(Clone)]
pub struct RequestContext {
    pub operator: Arc<OperatorConfig>,
    pub state: AppState,
}

impl RequestContext {
    /// Create a context with a specific operator.
    pub fn new(state: AppState, operator: Arc<OperatorConfig>) -> Self {
        Self { operator, state }
    }

    /// Create a context using the default operator.
    /// Convenience for USSD and other paths during incremental migration.
    pub async fn from_default(state: &AppState) -> Result<Self, AppError> {
        let operator = crate::operator::load_default(state).await?;
        Ok(Self {
            operator,
            state: state.clone(),
        })
    }

    /// Create a context by resolving an operator from a USSD shortcode.
    pub async fn from_shortcode(state: &AppState, shortcode: &str) -> Result<Self, AppError> {
        let operator = crate::operator::load_by_shortcode(state, shortcode).await?;
        Ok(Self {
            operator,
            state: state.clone(),
        })
    }

    /// Create a context by resolving an operator from its ID.
    pub async fn from_operator_id(
        state: &AppState,
        operator_id: uuid::Uuid,
    ) -> Result<Self, AppError> {
        let operator = crate::operator::load_by_id(state, operator_id).await?;
        Ok(Self {
            operator,
            state: state.clone(),
        })
    }
}
