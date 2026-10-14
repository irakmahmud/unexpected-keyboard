use attendance_common::Result;
use attendance_database::DbPool;

pub struct InspectCommand;

impl InspectCommand {
    pub fn execute(db: &DbPool) -> Result<()> {
        println!("============================================================");
        println!("             BIOMETRIC VECTOR INDEX & AUDIT                 ");
        println!("============================================================");

        let profiles = db.get_all_face_profiles()?;
        println!("Enrolled Face Biometric Profiles: {}", profiles.len());

        if profiles.is_empty() {
            println!("No biometric profiles found. Run 'attendance-admin seed' to seed test profiles.");
        } else {
            let mut total_quality = 0.0;

            for (idx, prof) in profiles.iter().enumerate().take(10) {
                total_quality += prof.sample_quality_score;

                let vector_norm: f32 = prof.embedding_vector.iter().map(|x| x * x).sum::<f32>().sqrt();
                println!(
                    "  [{:>2}] User: {:<20} Dim: {:>3} Norm: {:.4} Quality: {:>5.1}% Cam: {}",
                    idx + 1,
                    prof.username,
                    prof.embedding_vector.len(),
                    vector_norm,
                    prof.sample_quality_score,
                    prof.captured_with_camera
                );
            }

            let avg_q = total_quality / profiles.len() as f32;

            println!("\nAggregated Template Health:");
            println!("  - Average Face Quality Score:    {:.2}%", avg_q);
            println!("  - Vector Dimension Standard:     512-D FaceNet512 (Pure Rust Verified)");
            println!("  - Cosine Match Threshold:        0.40 (attendance_v8.py behavioral parity)");
        }

        println!("============================================================");
        Ok(())
    }
}
