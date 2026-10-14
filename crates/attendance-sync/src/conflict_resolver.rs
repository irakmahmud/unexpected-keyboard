use attendance_domain::AttendanceRecord;

pub struct ConflictResolver;

impl ConflictResolver {
    /// Resolve conflicting attendance logs: earliest in_time wins, latest out_time wins
    pub fn resolve_attendance_conflict(local: &AttendanceRecord, incoming: &AttendanceRecord) -> AttendanceRecord {
        let mut resolved = local.clone();

        // Earliest in-time
        if let (Some(l_in), Some(i_in)) = (&local.in_time, &incoming.in_time) {
            if i_in < l_in {
                resolved.in_time = Some(i_in.clone());
            }
        } else if resolved.in_time.is_none() {
            resolved.in_time = incoming.in_time.clone();
        }

        // Latest out-time
        if let (Some(l_out), Some(i_out)) = (&local.out_time, &incoming.out_time) {
            if i_out > l_out {
                resolved.out_time = Some(i_out.clone());
            }
        } else if resolved.out_time.is_none() {
            resolved.out_time = incoming.out_time.clone();
        }

        resolved
    }
}
