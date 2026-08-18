import { describe, it, expect } from "vitest";
import {
  isAlphabeticKeyboardKey,
} from "./keyboardKeyCase";
import {
  captureHotkey,
  captureMouseHotkey,
  formatHotkeyForDisplay,
} from "./hotkeys";

describe("keyboardKeyCase", () => {
  it("detects alphabetic keys", () => {
    expect(isAlphabeticKeyboardKey("a")).toBe(true);
    expect(isAlphabeticKeyboardKey("KeyA")).toBe(true);
    expect(isAlphabeticKeyboardKey("1")).toBe(false);
    expect(isAlphabeticKeyboardKey("Enter")).toBe(false);
  });
});

describe("hotkeys", () => {
  it("captures keyboard hotkey with modifiers", () => {
    const result = captureHotkey({
      key: "a",
      code: "KeyA",
      ctrlKey: true,
      altKey: false,
      shiftKey: false,
      metaKey: false,
    });
    expect(result).toBe("ctrl+KeyA");
  });

  it("captures mouse hotkey", () => {
    const result = captureMouseHotkey({
      button: 2,
      ctrlKey: true,
      altKey: false,
      shiftKey: false,
      metaKey: false,
    });
    expect(result).toBe("ctrl+mouseright");
  });

  it("ignores modifier-only keys", () => {
    expect(captureHotkey({
      key: "Control",
      code: "ControlLeft",
      ctrlKey: true,
      altKey: false,
      shiftKey: false,
      metaKey: false,
    })).toBeNull();
  });

  it("formats hotkey for display", () => {
    const formatted = formatHotkeyForDisplay("ctrl+shift+KeyA", null);
    expect(formatted).toBe("Ctrl + Shift + A");
  });

  it("formats mouse hotkey for display", () => {
    const formatted = formatHotkeyForDisplay("ctrl+mouseleft", null);
    expect(formatted).toBe("Ctrl + Mouse Left");
  });
});
