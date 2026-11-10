// The Rust root tested from TypeScript: `nsh test` runs this file in a
// headless runner it builds with the crate, and `open(".")` starts the root
// (`nsh.json` "root").
import { expect, open, test } from "nsh/test";

test("renders reminder app", async () => {
  const app = await open(".");
  expect(app.texts()).toContain("reminder 提醒");
  expect(app.texts()).toContain("喝水");
  expect(app.texts()).toContain("起身活动");
  expect(app.texts()).toContain("带薪拉屎");
});
