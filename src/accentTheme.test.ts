import { describe, it, expect } from "vitest";
import { applyAccentTheme } from "./accentTheme";

describe("applyAccentTheme", () => {
  it("sets CSS variables for dark theme", () => {
    applyAccentTheme("#ff5733", "dark");
    const root = document.documentElement;
    expect(root.style.getPropertyValue("--accent-green")).toBe("rgba(255, 87, 51, 0.75)");
    expect(root.style.getPropertyValue("--accent-green-strong")).toBe("rgba(255, 87, 51, 0.92)");
  });

  it("sets CSS variables for light theme", () => {
    applyAccentTheme("#00ff00", "light");
    const root = document.documentElement;
    expect(root.style.getPropertyValue("--accent-green")).toBe("rgba(0, 255, 0, 0.9)");
    expect(root.style.getPropertyValue("--accent-green-soft")).toBe("rgba(0, 255, 0, 0.2)");
  });
});
