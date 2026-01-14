use notify_rust::Notification;

fn main() {
    let mut notification = Notification::new();
    notification.summary("Test Icon");
    notification.body("Testing icon display");
    
    // Test 1: Use absolute path to embedded icon
    let icon_path = "/var/folders/p1/472yvqbx3zv7gdmp2hyyb2gm0000gn/T/claude-icon-99999.png";
    println!("Setting icon to: {}", icon_path);
    notification.icon(icon_path);
    
    let result = notification.show();
    println!("Result: {:?}", result);
}
