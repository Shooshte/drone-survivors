//! Campaign discoveries and the forfeitable reserve battery purchase.
use crate::economy::Amounts;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Secrets {
    pub blueprint: bool,
    pub reserve_battery: bool,
}

impl Secrets {
    pub fn discover_blueprint(&mut self) -> bool {
        if self.blueprint {
            return false;
        }
        self.blueprint = true;
        true
    }

    pub fn purchase(&mut self, wallet: &mut Amounts) -> Result<(), &'static str> {
        if !self.blueprint {
            return Err("Discover the Reserve battery blueprint first.");
        }
        if self.reserve_battery {
            return Err("Reserve battery is already active.");
        }
        wallet
            .try_spend(Amounts {
                salvage: 20,
                components: 1,
            })
            .map_err(|_| "Need 20 salvage and 1 component.")?;
        self.reserve_battery = true;
        Ok(())
    }

    pub fn lose_battery(&mut self) {
        self.reserve_battery = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blueprint_payment_and_single_active_purchase() {
        let mut secrets = Secrets::default();
        let mut wallet = Amounts {
            salvage: 20,
            components: 1,
        };
        assert!(secrets.purchase(&mut wallet).is_err());
        assert_eq!(wallet.salvage, 20);
        assert!(secrets.discover_blueprint());
        assert!(!secrets.discover_blueprint());
        wallet.components = 0;
        assert!(secrets.purchase(&mut wallet).is_err());
        assert_eq!(wallet.salvage, 20);
        wallet.components = 1;
        secrets.purchase(&mut wallet).unwrap();
        assert_eq!(wallet, Amounts::default());
        assert!(secrets.purchase(&mut wallet).is_err());
        secrets.lose_battery();
        assert!(secrets.blueprint);
        wallet = Amounts {
            salvage: 20,
            components: 1,
        };
        secrets.purchase(&mut wallet).unwrap();
    }
}
