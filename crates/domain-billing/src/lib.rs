//! 额度与成本领域：分配、预占、结算、归因、价格应用、对账和调整。
//!
//! 金额使用十进制定点表达，不使用 f64 作为财务权威值。
//! 精度、币种、舍入与周期时区在契约阶段冻结；此处只给出骨架。

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// ISO 4217 风格的币种代码（例如 CNY、USD）。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Currency(String);

#[derive(Debug, Error, PartialEq, Eq)]
pub enum BillingError {
    #[error("币种代码必须是 3 位大写字母")]
    InvalidCurrency,
    #[error("币种不一致：{0} 与 {1}")]
    CurrencyMismatch(String, String),
    #[error("金额不能为负")]
    NegativeAmount,
    #[error("额度不足：可用 {available}，申请 {requested}")]
    InsufficientQuota {
        available: Decimal,
        requested: Decimal,
    },
}

impl Currency {
    pub fn new(code: &str) -> Result<Self, BillingError> {
        if code.len() == 3 && code.chars().all(|c| c.is_ascii_uppercase()) {
            Ok(Self(code.to_string()))
        } else {
            Err(BillingError::InvalidCurrency)
        }
    }

    pub fn code(&self) -> &str {
        &self.0
    }
}

/// 非负金额。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    amount: Decimal,
    currency: Currency,
}

impl Money {
    pub fn new(amount: Decimal, currency: Currency) -> Result<Self, BillingError> {
        if amount.is_sign_negative() && !amount.is_zero() {
            return Err(BillingError::NegativeAmount);
        }
        Ok(Self { amount, currency })
    }

    pub fn amount(&self) -> Decimal {
        self.amount
    }

    pub fn currency(&self) -> &Currency {
        &self.currency
    }
}

/// 某一层级额度账户在一个周期内的快照（权威值保存在 PostgreSQL）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuotaSnapshot {
    pub limit: Money,
    pub consumed: Money,
    pub reserved: Money,
}

impl QuotaSnapshot {
    /// 检查“已消费 + 已占用 + 本次申请”不超过上限。
    /// 实际预占必须在事务内按稳定顺序锁定全部祖先路径后执行。
    pub fn check_reservation(&self, request: &Money) -> Result<(), BillingError> {
        for m in [&self.consumed, &self.reserved, request] {
            if m.currency != self.limit.currency {
                return Err(BillingError::CurrencyMismatch(
                    self.limit.currency.0.clone(),
                    m.currency.0.clone(),
                ));
            }
        }
        let available = self.limit.amount - self.consumed.amount - self.reserved.amount;
        if request.amount > available {
            return Err(BillingError::InsufficientQuota {
                available,
                requested: request.amount,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn cny(v: &str) -> Money {
        Money::new(Decimal::from_str(v).unwrap(), Currency::new("CNY").unwrap()).unwrap()
    }

    #[test]
    fn currency_and_amount_validation() {
        assert!(Currency::new("cny").is_err());
        assert!(Money::new(Decimal::from(-1), Currency::new("CNY").unwrap()).is_err());
    }

    #[test]
    fn reservation_respects_consumed_and_reserved() {
        let q = QuotaSnapshot {
            limit: cny("100.00"),
            consumed: cny("60.00"),
            reserved: cny("30.00"),
        };
        assert!(q.check_reservation(&cny("10.00")).is_ok());
        assert!(matches!(
            q.check_reservation(&cny("10.01")),
            Err(BillingError::InsufficientQuota { .. })
        ));
    }

    #[test]
    fn currency_mismatch_is_rejected() {
        let q = QuotaSnapshot {
            limit: cny("100"),
            consumed: cny("0"),
            reserved: cny("0"),
        };
        let usd = Money::new(Decimal::ONE, Currency::new("USD").unwrap()).unwrap();
        assert!(matches!(
            q.check_reservation(&usd),
            Err(BillingError::CurrencyMismatch(..))
        ));
    }
}
