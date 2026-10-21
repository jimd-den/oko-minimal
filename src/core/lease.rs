//! Who may present. One owner at a time, and no owner after a revocation.
//!
//! The console owns the screen at boot. It grants a lease to one session, and
//! the lease carries a generation so a late frame from a dead session cannot be
//! painted over a live one. Revocation is a state change, not a message: the
//! next `may_present` answers for the new state, and the stale holder has no
//! way back in.
//!
//! This is a state machine, not a lock. It holds no file descriptor, spawns no
//! process, and can be tested without a display.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnerId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Generation(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeaseError {
    AlreadyHeld,
    NoOwner,
    NotOwner,
    Stale,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Lease {
    holder: Option<(OwnerId, Generation)>,
    last_generation: u64,
}

impl Lease {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn holder(&self) -> Option<(OwnerId, Generation)> {
        self.holder
    }

    pub fn grant(&mut self, owner: OwnerId) -> Result<Generation, LeaseError> {
        if self.holder.is_some() {
            return Err(LeaseError::AlreadyHeld);
        }
        self.last_generation += 1;
        let generation = Generation(self.last_generation);
        self.holder = Some((owner, generation));
        Ok(generation)
    }

    pub fn revoke(&mut self) -> Option<Generation> {
        self.holder.take().map(|(_, generation)| generation)
    }

    pub fn may_present(&self, owner: OwnerId, generation: Generation) -> Result<(), LeaseError> {
        match self.holder {
            None => Err(LeaseError::NoOwner),
            Some((holder, _)) if holder != owner => Err(LeaseError::NotOwner),
            Some((_, current)) if current != generation => Err(LeaseError::Stale),
            Some(_) => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OWNER: OwnerId = OwnerId(1);

    #[test]
    fn granting_a_free_display_returns_a_generation() {
        let mut lease = Lease::new();

        assert_eq!(lease.grant(OWNER), Ok(Generation(1)));
        assert_eq!(lease.holder(), Some((OWNER, Generation(1))));
    }

    #[test]
    fn a_second_owner_cannot_take_a_held_display() {
        let mut lease = Lease::new();
        lease.grant(OWNER).expect("the first grant");

        assert_eq!(lease.grant(OwnerId(2)), Err(LeaseError::AlreadyHeld));
    }

    #[test]
    fn a_revoked_lease_cannot_present() {
        let mut lease = Lease::new();
        let generation = lease.grant(OWNER).expect("a grant");
        lease.revoke();

        assert_eq!(
            lease.may_present(OWNER, generation),
            Err(LeaseError::NoOwner)
        );
    }

    #[test]
    fn a_stale_generation_cannot_present_after_a_regrant() {
        let mut lease = Lease::new();
        let stale = lease.grant(OWNER).expect("a grant");
        lease.revoke();
        lease.grant(OwnerId(2)).expect("a regrant");

        assert_eq!(lease.may_present(OWNER, stale), Err(LeaseError::NotOwner));
        assert_eq!(lease.may_present(OwnerId(2), stale), Err(LeaseError::Stale));
    }

    #[test]
    fn the_holder_may_present_with_its_generation() {
        let mut lease = Lease::new();
        let generation = lease.grant(OWNER).expect("a grant");

        assert_eq!(lease.may_present(OWNER, generation), Ok(()));
    }

    #[test]
    fn revoking_a_free_display_does_nothing() {
        let mut lease = Lease::new();

        assert_eq!(lease.revoke(), None);
    }
}
