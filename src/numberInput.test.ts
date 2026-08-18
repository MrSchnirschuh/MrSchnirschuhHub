import { describe, it, expect } from "vitest";
import { normalizeIntegerRaw } from "./numberInput.js";

describe("normalizeIntegerRaw", () => {
  it("strips leading zeros from positive numbers", () => {
    expect(normalizeIntegerRaw("007")).toBe("7");
  });

  it("keeps a bare minus sign intact", () => {
    expect(normalizeIntegerRaw("-")).toBe("-");
  });

  it("strips leading zeros from negative numbers", () => {
    expect(normalizeIntegerRaw("-007")).toBe("-7");
  });

  it("returns zero as '0'", () => {
    expect(normalizeIntegerRaw("0")).toBe("0");
  });

  it("returns empty string for empty input", () => {
    expect(normalizeIntegerRaw("")).toBe("");
  });


});
