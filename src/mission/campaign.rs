//! Session campaign topology. Mission identities are independent of encounter content.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct MissionId(u8);

impl MissionId {
    pub const ALL: [Self; 12] = [
        Self(0),
        Self(1),
        Self(2),
        Self(3),
        Self(4),
        Self(5),
        Self(6),
        Self(7),
        Self(8),
        Self(9),
        Self(10),
        Self(11),
    ];

    pub fn index(self) -> usize {
        self.0 as usize
    }

    pub fn title(self) -> String {
        let role = match self.index() % 4 {
            0 => "Introduction",
            1 => "Branch A",
            2 => "Branch B",
            _ => "Finale",
        };
        format!(
            "Mission {:02} / Act {} / {role}",
            self.index() + 1,
            self.index() / 4 + 1
        )
    }

    pub fn prerequisites(self) -> &'static [Self] {
        let index = self.index();
        match index % 4 {
            0 if index == 0 => &[],
            0 => &Self::ALL[index - 1..index],
            1 => &Self::ALL[index - 1..index],
            2 => &Self::ALL[index - 2..index - 1],
            _ => &Self::ALL[index - 2..index],
        }
    }

    pub fn requirement(self) -> String {
        let ids = self
            .prerequisites()
            .iter()
            .map(|id| format!("{:02}", id.index() + 1))
            .collect::<Vec<_>>();
        if ids.is_empty() {
            "No prerequisites".into()
        } else {
            format!("Complete {}", ids.join(" + "))
        }
    }
}

#[derive(Default)]
pub(crate) struct Progress {
    completed: [bool; 12],
}
impl Progress {
    pub fn completed(&self, mission: MissionId) -> bool {
        self.completed[mission.index()]
    }
    pub fn unlocked(&self, mission: MissionId) -> bool {
        mission.prerequisites().iter().all(|id| self.completed(*id))
    }
    pub fn complete(&mut self, mission: MissionId) {
        if self.unlocked(mission) {
            self.completed[mission.index()] = true;
        }
    }
    pub fn count(&self) -> usize {
        self.completed.iter().filter(|done| **done).count()
    }
    pub fn finished(&self) -> bool {
        self.count() == MissionId::ALL.len()
    }
}
