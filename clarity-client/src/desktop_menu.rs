//! Desktop native menu integration
//!
//! This module provides native menu bar functionality for Dioxus Desktop applications.
//! It supports cross-platform menu creation with keyboard shortcuts and action handlers.

use std::collections::HashMap;

/// Menu errors
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuError {
  /// Invalid menu item
  InvalidItem(String),
  /// Invalid accelerator
  InvalidAccelerator(String),
  /// Menu creation failed
  CreationFailed(String),
  /// Menu action failed
  ActionFailed(String),
}

impl std::fmt::Display for MenuError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::InvalidItem(msg) => write!(f, "Invalid menu item: {msg}"),
      Self::InvalidAccelerator(msg) => write!(f, "Invalid accelerator: {msg}"),
      Self::CreationFailed(msg) => write!(f, "Menu creation failed: {msg}"),
      Self::ActionFailed(msg) => write!(f, "Menu action failed: {msg}"),
    }
  }
}

impl std::error::Error for MenuError {}

/// Keyboard accelerator (shortcut)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Accelerator {
  /// Key code (e.g., "q", "s", "f")
  pub key: String,
  /// Command key (Cmd on macOS, Ctrl on other platforms)
  pub cmd: bool,
  /// Shift key
  pub shift: bool,
  /// Alt/Option key
  pub alt: bool,
}

impl Accelerator {
  /// Create a new accelerator
  ///
  /// # Errors
  /// Returns `MenuError::InvalidAccelerator` if key is empty
  pub fn new(key: String) -> Result<Self, MenuError> {
    if key.is_empty() {
      return Err(MenuError::InvalidAccelerator(
        "Accelerator key cannot be empty".to_string(),
      ));
    }

    if key.len() > 1 {
      return Err(MenuError::InvalidAccelerator(format!(
        "Accelerator key must be a single character: {}",
        key
      )));
    }

    Ok(Self {
      key,
      cmd: false,
      shift: false,
      alt: false,
    })
  }

  /// Set Command/Ctrl modifier
  #[must_use]
  pub const fn with_cmd(mut self) -> Self {
    self.cmd = true;
    self
  }

  /// Set Shift modifier
  #[must_use]
  pub const fn with_shift(mut self) -> Self {
    self.shift = true;
    self
  }

  /// Set Alt/Option modifier
  #[must_use]
  pub const fn with_alt(mut self) -> Self {
    self.alt = true;
    self
  }

  /// Format accelerator for display (e.g., "Cmd+Q", "Ctrl+S")
  #[must_use]
  pub fn format(&self) -> String {
    let mut parts = Vec::new();

    if self.cmd {
      // Use "Cmd" on macOS, "Ctrl" on other platforms
      if cfg!(target_os = "macos") {
        parts.push("Cmd");
      } else {
        parts.push("Ctrl");
      }
    }

    if self.shift {
      parts.push("Shift");
    }

    if self.alt {
      // Use "Option" on macOS, "Alt" on other platforms
      if cfg!(target_os = "macos") {
        parts.push("Option");
      } else {
        parts.push("Alt");
      }
    }

    let key_upper = self.key.to_uppercase();
    parts.push(&key_upper);

    parts.join("+")
  }
}

/// Menu item type
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuItemType {
  /// Regular menu item
  Normal,
  /// Checkbox menu item
  Checkbox,
  /// Radio menu item
  Radio(String), // group name
  /// Separator
  Separator,
  /// Submenu
  Submenu(Vec<MenuItem>),
}

/// Menu item
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuItem {
  /// Unique item ID
  pub id: String,
  /// Item label
  pub label: String,
  /// Item type
  pub item_type: MenuItemType,
  /// Keyboard accelerator
  pub accelerator: Option<Accelerator>,
  /// Whether item is enabled
  pub enabled: bool,
  /// Whether item is checked (for checkbox/radio items)
  pub checked: bool,
}

impl MenuItem {
  /// Create a new menu item
  ///
  /// # Errors
  /// Returns `MenuError::InvalidItem` if id or label is empty
  pub fn new(id: String, label: String) -> Result<Self, MenuError> {
    if id.is_empty() {
      return Err(MenuError::InvalidItem(
        "Menu item ID cannot be empty".to_string(),
      ));
    }

    if label.is_empty() {
      return Err(MenuError::InvalidItem(
        "Menu item label cannot be empty".to_string(),
      ));
    }

    Ok(Self {
      id,
      label,
      item_type: MenuItemType::Normal,
      accelerator: None,
      enabled: true,
      checked: false,
    })
  }

  /// Set item type
  ///
  /// # Errors
  /// Returns `MenuError::InvalidItem` if item type is invalid
  #[must_use]
  pub fn with_type(mut self, item_type: MenuItemType) -> Self {
    self.item_type = item_type;
    self
  }

  /// Set keyboard accelerator
  #[must_use]
  pub fn with_accelerator(mut self, accelerator: Accelerator) -> Self {
    self.accelerator = Some(accelerator);
    self
  }

  /// Set enabled state
  #[must_use]
  pub fn with_enabled(mut self, enabled: bool) -> Self {
    self.enabled = enabled;
    self
  }

  /// Set checked state (for checkbox/radio items)
  #[must_use]
  pub fn with_checked(mut self, checked: bool) -> Self {
    self.checked = checked;
    self
  }
}

/// Menu bar
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuBar {
  /// Menu ID
  pub id: String,
  /// Menu label
  pub label: String,
  /// Menu items
  pub items: Vec<MenuItem>,
}

impl MenuBar {
  /// Create a new menu bar
  ///
  /// # Errors
  /// Returns `MenuError::InvalidItem` if id or label is empty
  pub fn new(id: String, label: String) -> Result<Self, MenuError> {
    if id.is_empty() {
      return Err(MenuError::InvalidItem(
        "Menu bar ID cannot be empty".to_string(),
      ));
    }

    if label.is_empty() {
      return Err(MenuError::InvalidItem(
        "Menu bar label cannot be empty".to_string(),
      ));
    }

    Ok(Self {
      id,
      label,
      items: Vec::new(),
    })
  }

  /// Add a menu item
  ///
  /// # Errors
  /// Returns `MenuError::InvalidItem` if item is invalid
  pub fn add_item(&mut self, item: MenuItem) -> Result<(), MenuError> {
    // Validate item
    if item.id.is_empty() {
      return Err(MenuError::InvalidItem(
        "Menu item ID cannot be empty".to_string(),
      ));
    }

    self.items.push(item);
    Ok(())
  }

  /// Add a separator
  pub fn add_separator(&mut self) {
    let separator = MenuItem {
      id: format!("separator_{}", self.items.len()),
      label: String::new(),
      item_type: MenuItemType::Separator,
      accelerator: None,
      enabled: true,
      checked: false,
    };
    self.items.push(separator);
  }

  /// Get menu item by ID
  #[must_use]
  pub fn get_item(&self, id: &str) -> Option<&MenuItem> {
    self.items.iter().find(|item| item.id == id)
  }

  /// Get mutable menu item by ID
  #[must_use]
  pub fn get_item_mut(&mut self, id: &str) -> Option<&mut MenuItem> {
    self.items.iter_mut().find(|item| item.id == id)
  }
}

/// Desktop menu manager
pub struct DesktopMenu {
  /// Menu bars
  menus: Vec<MenuBar>,
  /// Action handlers
  handlers: HashMap<String, Box<dyn Fn(&str) -> Result<(), MenuError> + Send + Sync>>,
}

impl DesktopMenu {
  /// Create a new desktop menu manager
  #[must_use]
  pub fn new() -> Self {
    Self {
      menus: Vec::new(),
      handlers: HashMap::new(),
    }
  }

  /// Add a menu bar
  ///
  /// # Errors
  /// Returns `MenuError::CreationFailed` if menu bar is invalid
  pub fn add_menu(&mut self, menu: MenuBar) -> Result<(), MenuError> {
    // Validate menu
    if menu.id.is_empty() {
      return Err(MenuError::CreationFailed(
        "Menu bar ID cannot be empty".to_string(),
      ));
    }

    self.menus.push(menu);
    Ok(())
  }

  /// Register an action handler for a menu item
  pub fn register_handler<F>(&mut self, item_id: String, handler: F) -> Result<(), MenuError>
  where
    F: Fn(&str) -> Result<(), MenuError> + Send + Sync + 'static,
  {
    if item_id.is_empty() {
      return Err(MenuError::InvalidItem(
        "Item ID cannot be empty".to_string(),
      ));
    }

    self.handlers.insert(item_id, Box::new(handler));
    Ok(())
  }

  /// Trigger a menu action
  ///
  /// # Errors
  /// Returns `MenuError::ActionFailed` if action handler fails
  pub fn trigger_action(&self, item_id: &str) -> Result<(), MenuError> {
    let handler = self.handlers.get(item_id).ok_or_else(|| {
      MenuError::ActionFailed(format!("No handler registered for item: {}", item_id))
    })?;

    handler(item_id)
  }

  /// Get all menu bars
  #[must_use]
  pub const fn menus(&self) -> &Vec<MenuBar> {
    &self.menus
  }

  /// Get menu bar by ID
  #[must_use]
  pub fn get_menu(&self, id: &str) -> Option<&MenuBar> {
    self.menus.iter().find(|menu| menu.id == id)
  }

  /// Get mutable menu bar by ID
  #[must_use]
  pub fn get_menu_mut(&mut self, id: &str) -> Option<&mut MenuBar> {
    self.menus.iter_mut().find(|menu| menu.id == id)
  }
}

impl Default for DesktopMenu {
  fn default() -> Self {
    Self::new()
  }
}

/// Create default application menu (macOS style)
///
/// # Errors
/// Returns `MenuError::CreationFailed` if menu creation fails
pub fn create_default_app_menu() -> Result<DesktopMenu, MenuError> {
  let mut menu = DesktopMenu::new();

  // Application menu (macOS)
  #[cfg(target_os = "macos")]
  {
    let mut app_menu = MenuBar::new("app".to_string(), "App".to_string())?;

    let about_item = MenuItem::new("about".to_string(), "About".to_string())?;
    app_menu.add_item(about_item)?;

    app_menu.add_separator();

    let preferences_item = MenuItem::new("preferences".to_string(), "Preferences...".to_string())?
      .with_accelerator(Accelerator::new(",".to_string())?.with_cmd());
    app_menu.add_item(preferences_item)?;

    app_menu.add_separator();

    let quit_item = MenuItem::new("quit".to_string(), "Quit".to_string())?
      .with_accelerator(Accelerator::new("q".to_string())?.with_cmd());
    app_menu.add_item(quit_item)?;

    menu.add_menu(app_menu)?;
  }

  // File menu
  let mut file_menu = MenuBar::new("file".to_string(), "File".to_string())?;

  let new_item = MenuItem::new("new".to_string(), "New".to_string())?
    .with_accelerator(Accelerator::new("n".to_string())?.with_cmd());
  file_menu.add_item(new_item)?;

  let open_item = MenuItem::new("open".to_string(), "Open...".to_string())?
    .with_accelerator(Accelerator::new("o".to_string())?.with_cmd());
  file_menu.add_item(open_item)?;

  file_menu.add_separator();

  let save_item = MenuItem::new("save".to_string(), "Save".to_string())?
    .with_accelerator(Accelerator::new("s".to_string())?.with_cmd());
  file_menu.add_item(save_item)?;

  let save_as_item = MenuItem::new("save_as".to_string(), "Save As...".to_string())?
    .with_accelerator(Accelerator::new("s".to_string())?.with_cmd().with_shift());
  file_menu.add_item(save_as_item)?;

  menu.add_menu(file_menu)?;

  // Edit menu
  let mut edit_menu = MenuBar::new("edit".to_string(), "Edit".to_string())?;

  let undo_item = MenuItem::new("undo".to_string(), "Undo".to_string())?
    .with_accelerator(Accelerator::new("z".to_string())?.with_cmd());
  edit_menu.add_item(undo_item)?;

  let redo_item = MenuItem::new("redo".to_string(), "Redo".to_string())?
    .with_accelerator(Accelerator::new("z".to_string())?.with_cmd().with_shift());
  edit_menu.add_item(redo_item)?;

  edit_menu.add_separator();

  let cut_item = MenuItem::new("cut".to_string(), "Cut".to_string())?
    .with_accelerator(Accelerator::new("x".to_string())?.with_cmd());
  edit_menu.add_item(cut_item)?;

  let copy_item = MenuItem::new("copy".to_string(), "Copy".to_string())?
    .with_accelerator(Accelerator::new("c".to_string())?.with_cmd());
  edit_menu.add_item(copy_item)?;

  let paste_item = MenuItem::new("paste".to_string(), "Paste".to_string())?
    .with_accelerator(Accelerator::new("v".to_string())?.with_cmd());
  edit_menu.add_item(paste_item)?;

  let select_all_item = MenuItem::new("select_all".to_string(), "Select All".to_string())?
    .with_accelerator(Accelerator::new("a".to_string())?.with_cmd());
  edit_menu.add_item(select_all_item)?;

  menu.add_menu(edit_menu)?;

  Ok(menu)
}

#[cfg(test)]
mod tests {
  use super::*;

  // Martin Fowler Test Suite: Desktop Menu Management

  #[test]
  fn test_accelerator_new_valid() {
    // GIVEN: valid single-character key
    let key = "q".to_string();

    // WHEN: creating accelerator
    let result = Accelerator::new(key.clone());

    // THEN: accelerator should be created successfully
    assert!(result.is_ok());
    let acc = result.unwrap();
    assert_eq!(acc.key, key);
    assert_eq!(acc.cmd, false);
    assert_eq!(acc.shift, false);
    assert_eq!(acc.alt, false);
  }

  #[test]
  fn test_accelerator_new_empty_key() {
    // GIVEN: empty key
    let key = String::new();

    // WHEN: creating accelerator
    let result = Accelerator::new(key);

    // THEN: accelerator creation should fail
    assert!(result.is_err());
    assert!(matches!(result, Err(MenuError::InvalidAccelerator(_))));
  }

  #[test]
  fn test_accelerator_new_multi_char_key() {
    // GIVEN: multi-character key
    let key = "abc".to_string();

    // WHEN: creating accelerator
    let result = Accelerator::new(key);

    // THEN: accelerator creation should fail
    assert!(result.is_err());
    assert!(matches!(result, Err(MenuError::InvalidAccelerator(_))));
  }

  #[test]
  fn test_accelerator_with_cmd() {
    // GIVEN: valid key
    let acc = Accelerator::new("s".to_string()).unwrap();

    // WHEN: adding Cmd modifier
    let acc = acc.with_cmd();

    // THEN: cmd should be true
    assert!(acc.cmd);
  }

  #[test]
  fn test_accelerator_with_shift() {
    // GIVEN: valid key
    let acc = Accelerator::new("s".to_string()).unwrap();

    // WHEN: adding Shift modifier
    let acc = acc.with_shift();

    // THEN: shift should be true
    assert!(acc.shift);
  }

  #[test]
  fn test_accelerator_with_alt() {
    // GIVEN: valid key
    let acc = Accelerator::new("s".to_string()).unwrap();

    // WHEN: adding Alt modifier
    let acc = acc.with_alt();

    // THEN: alt should be true
    assert!(acc.alt);
  }

  #[test]
  fn test_accelerator_format_cmd() {
    // GIVEN: accelerator with Cmd
    let acc = Accelerator::new("q".to_string()).unwrap().with_cmd();

    // WHEN: formatting accelerator
    let formatted = acc.format();

    // THEN: should show "Cmd+Q" or "Ctrl+Q" depending on platform
    assert!(formatted.contains("Q"));
    if cfg!(target_os = "macos") {
      assert!(formatted.contains("Cmd"));
    } else {
      assert!(formatted.contains("Ctrl"));
    }
  }

  #[test]
  fn test_menu_item_new_valid() {
    // GIVEN: valid id and label
    let id = "quit".to_string();
    let label = "Quit".to_string();

    // WHEN: creating menu item
    let result = MenuItem::new(id.clone(), label.clone());

    // THEN: menu item should be created successfully
    assert!(result.is_ok());
    let item = result.unwrap();
    assert_eq!(item.id, id);
    assert_eq!(item.label, label);
    assert_eq!(item.enabled, true);
    assert_eq!(item.checked, false);
  }

  #[test]
  fn test_menu_item_new_empty_id() {
    // GIVEN: empty id
    let id = String::new();
    let label = "Quit".to_string();

    // WHEN: creating menu item
    let result = MenuItem::new(id, label);

    // THEN: menu item creation should fail
    assert!(result.is_err());
    assert!(matches!(result, Err(MenuError::InvalidItem(_))));
  }

  #[test]
  fn test_menu_item_new_empty_label() {
    // GIVEN: empty label
    let id = "quit".to_string();
    let label = String::new();

    // WHEN: creating menu item
    let result = MenuItem::new(id, label);

    // THEN: menu item creation should fail
    assert!(result.is_err());
    assert!(matches!(result, Err(MenuError::InvalidItem(_))));
  }

  #[test]
  fn test_menu_item_with_enabled() {
    // GIVEN: valid menu item
    let item = MenuItem::new("save".to_string(), "Save".to_string()).unwrap();

    // WHEN: setting enabled to false
    let item = item.with_enabled(false);

    // THEN: enabled should be false
    assert!(!item.enabled);
  }

  #[test]
  fn test_menu_item_with_checked() {
    // GIVEN: valid menu item
    let item = MenuItem::new("checkbox".to_string(), "Toggle".to_string())
      .unwrap()
      .with_type(MenuItemType::Checkbox);

    // WHEN: setting checked to true
    let item = item.with_checked(true);

    // THEN: checked should be true
    assert!(item.checked);
  }

  #[test]
  fn test_menu_bar_new_valid() {
    // GIVEN: valid id and label
    let id = "file".to_string();
    let label = "File".to_string();

    // WHEN: creating menu bar
    let result = MenuBar::new(id.clone(), label.clone());

    // THEN: menu bar should be created successfully
    assert!(result.is_ok());
    let menu = result.unwrap();
    assert_eq!(menu.id, id);
    assert_eq!(menu.label, label);
    assert!(menu.items.is_empty());
  }

  #[test]
  fn test_menu_bar_add_item() {
    // GIVEN: valid menu bar and item
    let mut menu = MenuBar::new("file".to_string(), "File".to_string()).unwrap();
    let item = MenuItem::new("new".to_string(), "New".to_string()).unwrap();

    // WHEN: adding item to menu
    let result = menu.add_item(item);

    // THEN: item should be added successfully
    assert!(result.is_ok());
    assert_eq!(menu.items.len(), 1);
  }

  #[test]
  fn test_menu_bar_add_separator() {
    // GIVEN: valid menu bar
    let mut menu = MenuBar::new("file".to_string(), "File".to_string()).unwrap();

    // WHEN: adding separator
    menu.add_separator();

    // THEN: separator should be added
    assert_eq!(menu.items.len(), 1);
    assert!(matches!(menu.items[0].item_type, MenuItemType::Separator));
  }

  #[test]
  fn test_menu_bar_get_item() {
    // GIVEN: menu bar with items
    let mut menu = MenuBar::new("file".to_string(), "File".to_string()).unwrap();
    let item = MenuItem::new("save".to_string(), "Save".to_string()).unwrap();
    menu.add_item(item).ok();

    // WHEN: getting item by id
    let found = menu.get_item("save");

    // THEN: item should be found
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, "save");
  }

  #[test]
  fn test_desktop_menu_new() {
    // GIVEN: no parameters
    // WHEN: creating desktop menu
    let menu = DesktopMenu::new();

    // THEN: should be created with empty menus and handlers
    assert!(menu.menus().is_empty());
  }

  #[test]
  fn test_desktop_menu_default() {
    // GIVEN: no parameters
    // WHEN: creating default desktop menu
    let menu = DesktopMenu::default();

    // THEN: should be created with empty menus
    assert!(menu.menus().is_empty());
  }

  #[test]
  fn test_desktop_menu_add_menu() {
    // GIVEN: desktop menu and menu bar
    let mut desktop_menu = DesktopMenu::new();
    let menu_bar = MenuBar::new("file".to_string(), "File".to_string()).unwrap();

    // WHEN: adding menu bar to desktop menu
    let result = desktop_menu.add_menu(menu_bar);

    // THEN: menu bar should be added successfully
    assert!(result.is_ok());
    assert_eq!(desktop_menu.menus().len(), 1);
  }

  #[test]
  fn test_desktop_menu_register_handler() {
    // GIVEN: desktop menu
    let mut desktop_menu = DesktopMenu::new();

    // WHEN: registering handler
    let result = desktop_menu.register_handler("quit".to_string(), |_| Ok(()));

    // THEN: handler should be registered successfully
    assert!(result.is_ok());
  }

  #[test]
  fn test_desktop_menu_trigger_action() {
    // GIVEN: desktop menu with registered handler
    let mut desktop_menu = DesktopMenu::new();
    let called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let called_clone = called.clone();

    desktop_menu
      .register_handler("quit".to_string(), move |_| {
        called_clone.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
      })
      .ok();

    // WHEN: triggering action
    let result = desktop_menu.trigger_action("quit");

    // THEN: handler should be called
    assert!(result.is_ok());
    assert!(called.load(std::sync::atomic::Ordering::SeqCst));
  }

  #[test]
  fn test_create_default_app_menu() {
    // GIVEN: no parameters
    // WHEN: creating default app menu
    let result = create_default_app_menu();

    // THEN: default menu should be created successfully
    assert!(result.is_ok());
    let menu = result.unwrap();

    // Should have File and Edit menus (and App menu on macOS)
    assert!(menu.menus().len() >= 2);

    // Should have File menu
    assert!(menu.get_menu("file").is_some());

    // Should have Edit menu
    assert!(menu.get_menu("edit").is_some());
  }

  #[test]
  fn test_menu_error_display() {
    // GIVEN: various menu errors
    let err1 = MenuError::InvalidItem("Invalid".to_string());
    let err2 = MenuError::InvalidAccelerator("Bad accelerator".to_string());
    let err3 = MenuError::CreationFailed("Creation failed".to_string());
    let err4 = MenuError::ActionFailed("Action failed".to_string());

    // WHEN: converting errors to string
    let msg1 = err1.to_string();
    let msg2 = err2.to_string();
    let msg3 = err3.to_string();
    let msg4 = err4.to_string();

    // THEN: error messages should be descriptive
    assert!(msg1.contains("Invalid menu item"));
    assert!(msg2.contains("Invalid accelerator"));
    assert!(msg3.contains("Menu creation failed"));
    assert!(msg4.contains("Menu action failed"));
  }
}
