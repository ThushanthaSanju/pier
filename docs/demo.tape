# pier demo recording script
#
# To regenerate:
#   cargo build --release
#   vhs docs/demo.tape
#
# Run from the repository root.

Output docs/demo.gif

Set Shell        "bash"
Set FontSize     16
Set Width        1100
Set Height       640
Set Theme        "Catppuccin Mocha"
Set TypingSpeed  55ms
Set Padding      24

# ── Hidden bootstrap ───────────────────────────────────────────────────────
Env PIER_CONFIG_DIR "/tmp/pier-vhs-config"

Hide
Type "rm -rf /tmp/pier-vhs-config /tmp/pier-demo"
Enter
Type "mkdir -p /tmp/pier-vhs-config /tmp/pier-demo/acme-backend /tmp/pier-demo/acme-web /tmp/pier-demo/personal-blog"
Enter
Type "export PATH=$PWD/target/release:$PATH"
Enter
Type "cd /tmp/pier-demo/acme-backend"
Enter
Type "clear"
Enter
Sleep 400ms
Show

# ── Visible demo ───────────────────────────────────────────────────────────
Sleep 800ms
Type "# pier — dock at any project in one command"
Sleep 500ms
Enter
Sleep 600ms

Type "pj register"
Enter
Sleep 1600ms

Type "pj register /tmp/pier-demo/acme-web"
Enter
Sleep 1600ms

Type "pj register /tmp/pier-demo/personal-blog --name blog"
Enter
Sleep 1600ms

Enter
Type "# list — recency-sorted, with branch + last accessed"
Enter
Sleep 600ms
Type "pj list"
Enter
Sleep 2800ms

Enter
Type "# in your shell, the wrapper from 'pj init bash' eval's a script"
Enter
Type "# like this — that's how the cd survives into your parent shell:"
Enter
Sleep 800ms
Type "pj __shell --shell bash acme-backend"
Enter
Sleep 2800ms

Enter
Type "# substring matching just works"
Enter
Sleep 600ms
Type "pj __shell --shell bash web"
Enter
Sleep 2400ms

Sleep 1500ms
