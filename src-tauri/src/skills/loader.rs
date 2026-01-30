use serde::Deserialize;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Represents a loaded skill from a markdown file
#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub instruction: String, // The markdown body (prompt text)
}

/// YAML frontmatter structure
#[derive(Debug, Deserialize)]
struct SkillFrontmatter {
    name: String,
    description: String,
    #[serde(default)]
    trigger_keywords: Vec<String>,
}

/// Parse a skill file, separating YAML frontmatter from the body
fn parse_skill_file(content: &str) -> Option<Skill> {
    // Check if file starts with frontmatter delimiter
    if !content.starts_with("---") {
        return None;
    }

    // Find the closing frontmatter delimiter
    let rest = &content[3..]; // Skip opening ---
    let end_pos = rest.find("\n---")?;
    
    let yaml_content = rest[..end_pos].trim();
    let body = rest[end_pos + 4..].trim(); // Skip closing --- and newline

    // Parse YAML frontmatter
    let frontmatter: SkillFrontmatter = serde_yaml::from_str(yaml_content).ok()?;

    Some(Skill {
        name: frontmatter.name,
        description: frontmatter.description,
        instruction: body.to_string(),
    })
}

/// Load all skills from the specified directory
/// Recursively scans for .md files and parses their frontmatter
pub fn load_skills(skills_dir: &Path) -> Vec<Skill> {
    let mut skills = Vec::new();

    if !skills_dir.exists() {
        eprintln!("Skills directory does not exist: {:?}", skills_dir);
        return skills;
    }

    for entry in WalkDir::new(skills_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
    {
        let path = entry.path();
        
        match fs::read_to_string(path) {
            Ok(content) => {
                if let Some(skill) = parse_skill_file(&content) {
                    skills.push(skill);
                }
            }
            Err(e) => {
                eprintln!("Failed to read skill file {:?}: {}", path, e);
            }
        }
    }

    skills
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_skill_file() {
        let content = r#"---
name: "Test Skill"
description: "A test skill"
trigger_keywords: ["test"]
---
This is the instruction body."#;

        let skill = parse_skill_file(content).expect("Should parse skill");
        assert_eq!(skill.name, "Test Skill");
        assert_eq!(skill.description, "A test skill");
        assert_eq!(skill.instruction, "This is the instruction body.");
    }
}
