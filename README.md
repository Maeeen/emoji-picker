# Emoji Picker

This is a simple emoji picker that replaces the emoji picker in Windows ≥10.

## Goal

This executable should behave in the same manner as the Windows Emoji Picker, with the main goal of:
- having the same shortcut (achieved by using a `WH_KEYBOARD_LL` Windows hook)
- having the same behavior : focus to the main window is not lost when the picker is opened, and the picker closes when the focus is lost.

## Targets

* The main target is Windows 11, but it should work on older versions of Windows as well. No need for administrator privileges.
* On Linux/macOS, there is no need for this but the missing implementations are:
  * [ ] The keyboard shortcut to open the picker. (<kbd>Win</kbd> + <kbd>.</kbd>) (`src/key_shortcut.rs`)
  * [ ] The “key interceptor” that prevents focus being made to the Emoji Picker window AND intercepts every key made while the picker is open.

## Features

* Replaces the painfully slow and bloated Windows emoji picker (personal opinion.)
* You can tweak it to your liking.

## Caveats

There is many caveats when programming this kind of Windows application in Rust:
- a shit ton of `unsafe` tags

## TO-DO

* [ ] Lost focus closes the picker
* [ ] Customize emoji image source
* [ ] Customize shortcuts