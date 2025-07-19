use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct Config {
  pub groups: HashMap<String, Vec<String>>,
}

impl Config {
  pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
    let home = std::env::var("HOME")?;
    let config_path = format!("{}/.mssh_groups", home);

    if !Path::new(&config_path).exists() {
      return Err(format!("Config file not found: {}", config_path).into());
    }

    let content = fs::read_to_string(&config_path)?;
    let groups = Self::parse_groups(&content)?;

    Ok(Config { groups })
  }

  fn parse_groups(
    content: &str,
  ) -> Result<HashMap<String, Vec<String>>, Box<dyn std::error::Error>> {
    let mut groups = HashMap::new();
    let mut current_group = None;

    for line in content.lines() {
      let line = line.trim();
      if line.is_empty() || line.starts_with('#') {
        continue;
      }

      if line.starts_with('[') && line.ends_with(']') {
        let group_name = line[1..line.len() - 1].to_string();
        current_group = Some(group_name);
      } else if let Some(group_name) = &current_group {
        let servers = groups.entry(group_name.clone()).or_insert_with(Vec::new);
        servers.push(line.to_string());
      }
    }

    Ok(groups)
  }

  pub fn get_servers_resolved(&self, group_name: &str) -> Option<Vec<String>> {
    let mut resolved_servers = Vec::new();
    let mut visited = std::collections::HashSet::new();

    // Try to find the group with or without :g suffix
    let group_key = if self.groups.contains_key(group_name) {
      group_name.to_string()
    } else {
      format!("{}:g", group_name)
    };

    self.resolve_group(&group_key, &mut resolved_servers, &mut visited)
  }

  fn resolve_group(
    &self,
    group_name: &str,
    resolved_servers: &mut Vec<String>,
    visited: &mut std::collections::HashSet<String>,
  ) -> Option<Vec<String>> {
    if visited.contains(group_name) {
      return Some(resolved_servers.clone());
    }
    visited.insert(group_name.to_string());

    let group_servers = self.groups.get(group_name)?;

    for server in group_servers {
      if self.groups.contains_key(server) {
        // This is a group reference (exists as a group)
        self.resolve_group(server, resolved_servers, visited);
      } else {
        // This is a server, add it to the list
        resolved_servers.push(server.clone());
      }
    }

    Some(resolved_servers.clone())
  }
}
