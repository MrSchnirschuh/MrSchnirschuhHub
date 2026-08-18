import { describe, it, expect } from "vitest";
import {
  DEFAULT_ACCENT_COLOR,
  MAX_PRESETS,
  PRESET_NAME_MAX_LENGTH,
  DEFAULT_MAX_CLICK_SPEED,
  EXTENDED_MAX_CLICK_SPEED,
  CLICK_INTERVAL_OPTIONS,
  MODE_OPTIONS,
  MOUSE_BUTTON_OPTIONS,
  THEME_OPTIONS,
  TIME_LIMIT_UNIT_OPTIONS,
} from "./settingsSchema";

describe("settingsSchema constants", () => {
  it("exports sensible default constants", () => {
    expect(DEFAULT_ACCENT_COLOR).toBe("#22c55e");
    expect(MAX_PRESETS).toBe(20);
    expect(PRESET_NAME_MAX_LENGTH).toBe(40);
    expect(DEFAULT_MAX_CLICK_SPEED).toBe(500);
    expect(EXTENDED_MAX_CLICK_SPEED).toBe(1000);
  });

  it("exposes the expected option lists", () => {
    expect(CLICK_INTERVAL_OPTIONS.map((o) => o.value)).toEqual(["s", "m", "h", "d"]);
    expect(MODE_OPTIONS).toEqual(["Toggle", "Hold"]);
    expect(MOUSE_BUTTON_OPTIONS).toEqual(["Left", "Middle", "Right"]);
    expect(THEME_OPTIONS).toEqual(["dark", "light"]);
    expect(TIME_LIMIT_UNIT_OPTIONS).toEqual(["s", "m", "h"]);
  });
});
