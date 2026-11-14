#!/usr/bin/env bash
set -e

echo "==> Configuring DropBar for Hyprland..."

DROPBAR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HYPR_CONF_DIR="$HOME/.config/hypr"
DROPBAR_TARGET_CONF="$HYPR_CONF_DIR/dropbar.conf"
KEYBINDS_LUA="$HYPR_CONF_DIR/config/keybinds.lua"
SETTINGS_LUA="$HYPR_CONF_DIR/config/settings.lua"
DOWNLOAD_DIR="$HOME/Downloads/DropBar"

# 1. Create downloads folder
mkdir -p "$DOWNLOAD_DIR"
echo "  [✓] Created Download Shelf at: $DOWNLOAD_DIR"

# 2. Check if using Lua-based Hyprland config
if [ -f "$KEYBINDS_LUA" ]; then
    if ! grep -q "dropbar-desktop" "$KEYBINDS_LUA"; then
        echo "" >> "$KEYBINDS_LUA"
        echo "-- DropBar Launch Keybind (Super + T)" >> "$KEYBINDS_LUA"
        echo "hl.bind(mainMod .. \" + T\", hl.dsp.exec_cmd(\"$DROPBAR_DIR/target/debug/dropbar-desktop\"))" >> "$KEYBINDS_LUA"
        echo "  [✓] Added 'Super + T' keybind to $KEYBINDS_LUA"
    else
        echo "  [✓] 'Super + T' keybind already present in $KEYBINDS_LUA"
    fi

    if [ -f "$SETTINGS_LUA" ] && ! grep -q "dropbar-desktop" "$SETTINGS_LUA"; then
        echo "" >> "$SETTINGS_LUA"
        echo "-- DropBar Floating Shelf Rules" >> "$SETTINGS_LUA"
        echo "hl.window_rule({ name = \"dropbar-floating\", match = { class = \"dropbar-desktop\" }, float = true, pin = true })" >> "$SETTINGS_LUA"
        echo "  [✓] Added floating window rule to $SETTINGS_LUA"
    fi
fi

# 3. Also maintain dropbar.conf for standard hyprland.conf setups
cp -f "$DROPBAR_DIR/hyprland/dropbar.conf" "$DROPBAR_TARGET_CONF"
echo "  [✓] Installed dropbar.conf at: $DROPBAR_TARGET_CONF"

echo ""
echo "✨ DropBar Hyprland setup complete!"
echo "Press Super + T in Hyprland to launch your DropBar shelf."
