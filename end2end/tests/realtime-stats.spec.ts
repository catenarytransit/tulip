import { test, expect } from "@playwright/test";

const baseURL = process.env.TULIP_TEST_URL || "http://127.0.0.1:7914";

test("polls stats every second and keeps null timestamps distinct from zero counts", async ({ page }) => {
  let polls = 0;
  const timestamp = Math.floor(Date.now() / 1000) - 10;
  await page.route("**/api/get_gtfs_rt_stats", async (route) => {
    polls++;
    await route.fulfill({
      contentType: "application/json",
      body: JSON.stringify({
        timestamps: { Ok: { vehicle_positions: timestamp, trip_updates: null, alerts: timestamp } },
        chateaux: { Ok: [{
          chateau: { chateau: "test-chateau", realtime_feeds: ["f-test", "f-other"], schedule_feeds: ["f-schedule"], languages_avaliable: ["en"] },
          entities: { Ok: { vehicles: 0, trip_updates: 42 } },
          authoritative: { Ok: { vehicle_positions: 0 } },
        }] },
      }),
    });
  });
  await page.goto(`${baseURL}/gtfs-rt-stats?feed_id=f-test`);
  await expect(page.getByRole("heading", { name: "GTFS-RT stats", exact: true })).toBeVisible();
  await expect(page.getByRole("row", { name: /Trip updates/ })).toContainText("null (unavailable)");
  await expect(page.getByRole("row", { name: /Vehicle positions/ }).first()).toContainText(/\d+ s ago/);
  await expect(page.getByText("Counts cover all realtime feeds", { exact: false })).toBeVisible();
  await expect(page.getByText("Realtime feeds: f-test, f-other")).toBeVisible();
  await expect(page.getByRole("row", { name: "vehicles 0", exact: true })).toBeVisible();
  await expect.poll(() => polls).toBeGreaterThanOrEqual(3);
});

test("shows endpoint failures without replacing them with empty counters", async ({ page }) => {
  await page.route("**/api/get_gtfs_rt_stats", (route) => route.fulfill({
    contentType: "application/json",
    body: JSON.stringify({
      timestamps: { Err: "Birch returned 503" },
      chateaux: { Err: "Directory unavailable" },
    }),
  }));
  await page.goto(`${baseURL}/gtfs-rt-stats?feed_id=f-missing`);
  await expect(page.getByRole("alert").filter({ hasText: "Timestamps unavailable" })).toContainText("503");
  await expect(page.getByRole("alert").filter({ hasText: "Chateau metadata unavailable" })).toBeVisible();
});

test("requires a feed ID before starting polling", async ({ page }) => {
  let polls = 0;
  await page.route("**/api/get_gtfs_rt_stats", (route) => { polls++; return route.abort(); });
  await page.goto(`${baseURL}/gtfs-rt-stats`);
  await expect(page.getByRole("alert")).toContainText("Provide a feed ID");
  expect(polls).toBe(0);
});

test("skips overlapping polls and stops when the feed ID is removed", async ({ page }) => {
  let active = 0;
  let maxActive = 0;
  let polls = 0;
  await page.route("**/api/get_gtfs_rt_stats", async (route) => {
    polls++;
    active++;
    maxActive = Math.max(maxActive, active);
    await new Promise((resolve) => setTimeout(resolve, 2200));
    active--;
    await route.fulfill({
      contentType: "application/json",
      body: JSON.stringify({
        timestamps: { Ok: { vehicle_positions: 100, trip_updates: null, alerts: null } },
        chateaux: { Ok: [] },
      }),
    });
  });
  await page.goto(`${baseURL}/gtfs-rt-stats?feed_id=f-slow`);
  await expect(page.getByRole("row", { name: /Vehicle positions/ })).toBeVisible();
  expect(maxActive).toBe(1);
  expect(polls).toBe(1);
  // Exercise the router's query-string update without reloading the document.
  await page.evaluate(() => {
    window.history.pushState(null, "", "/gtfs-rt-stats");
    window.dispatchEvent(new PopStateEvent("popstate"));
  });
  await expect(page.getByRole("alert")).toContainText("Provide a feed ID");
  await page.waitForTimeout(1200);
  expect(polls).toBe(1);
});
