ALTER TABLE resume_profiles ADD COLUMN target_direction TEXT NOT NULL DEFAULT '';
ALTER TABLE resume_profiles ADD COLUMN notes TEXT NOT NULL DEFAULT '';
ALTER TABLE resume_profiles ADD COLUMN parent_profile_id TEXT REFERENCES resume_profiles(id);
ALTER TABLE resume_profiles ADD COLUMN archived_at TEXT;

ALTER TABLE applications ADD COLUMN resume_profile_id TEXT REFERENCES resume_profiles(id);

CREATE INDEX resume_profiles_parent_idx ON resume_profiles(parent_profile_id);
CREATE INDEX applications_resume_profile_idx ON applications(resume_profile_id);
