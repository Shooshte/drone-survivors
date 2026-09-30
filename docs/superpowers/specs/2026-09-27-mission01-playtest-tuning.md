# Mission 01 playtest tuning

The user approved the six proposed feedback changes and requested incremental
spawn escalation after payload pickup. This supersedes the original travel and
pacing defaults for Mission 01 only. Work continues in the existing isolated
playtest workspace.

- Retain scale 12 (53.37s measured rotor travel) and Scout speed 420u/s.
- Mission 01 chasers use a 340u/s cap. Beam multiplier remains 0.6, giving
  chasers an 88u/s nominal closing advantage over a slowed Scout.
- A small patrol of four chasers requests telegraphed spawns after three active
  seconds, then every eight active seconds throughout the attempt. Candidates
  are about 550–850u from the player's current position, with a safe body gap,
  terrain/arena/occupant clearance and navigation connectivity. Existing warning
  activation rechecks safety. These patrols supply pressure before pickup and
  during travel; first contact should occur about 5–8 seconds after launch.
- Fixed encounters retain exact counts. Pickup immediately enables the two
  permanent sources, each with four enemies per batch. Source interval is 5s
  before 20s of carrying, 4s at 20–40s, 3s at 40–60s, and 2s thereafter.
  No timer failure. All live enemies and warnings share cap96. Fixed requests
  persist; sources/patrols coalesce missed demand with bounded queues. Admission
  must remain fair. Pause freezes clocks; restart clears warnings and escalation.
- Mission 01 salvage attracts within 300 horizontal units at any legal flight
  height, with clear sight and swept path detection for flybys. Attraction moves
  the pickup through the ordinary collection/settlement path; walls still block
  attraction and collection. Other missions keep their existing pickup behavior.
- Holdout radius becomes840. Before entry, readable guidance states optional,
  stay inside30s, reward5components, and exit forfeits. During the attempt show
  a prominent countdown/progress and the reward; terminal states clearly say
  earned or forfeited. Existing one-attempt, pause, reset and settlement rules stay.
- Validate ordinary movement versus camping with real combat, plus source ramp,
  cap/fairness, warning safety, pickup height/flyby/walls, mission isolation and
  holdout guidance at1120x720/640x480. Do not equate scripted checks with human
  acceptance. Record measured behavior and any tuning changes.
