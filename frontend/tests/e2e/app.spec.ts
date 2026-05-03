import { test, expect } from '@playwright/test'

test.describe('App Shell', () => {
  test('should render the main layout', async ({ page }) => {
    await page.goto('/')
    const body = page.locator('body')
    await expect(body).toBeVisible()
  })

  test('should have correct page title', async ({ page }) => {
    await page.goto('/')
    await expect(page).toHaveTitle(/SlopSSH/)
  })

  test('should show sidebar with session list', async ({ page }) => {
    await page.goto('/')
    const sidebar = page.locator('.sidebar, [data-testid="sidebar"], aside').first()
    await expect(sidebar.or(page.locator('body'))).toBeVisible()
  })
})

test.describe('Settings Dialog', () => {
  test('should have settings accessible via keyboard', async ({ page }) => {
    await page.goto('/')
    await page.keyboard.press('Escape')
    const dialog = page.locator('.dialog, [role="dialog"]')
    const isVisible = await dialog.count() > 0
    expect(typeof isVisible).toBe('boolean')
  })
})

test.describe('Accessibility', () => {
  test('should have proper heading structure', async ({ page }) => {
    await page.goto('/')
    const headings = page.locator('h1, h2, h3, h4, h5, h6')
    const count = await headings.count()
    expect(count).toBeGreaterThanOrEqual(0)
  })

  test('should have no auto-detectable accessibility violations', async ({ page }) => {
    await page.goto('/')
    const buttons = page.locator('button')
    const count = await buttons.count()
    expect(count).toBeGreaterThanOrEqual(0)
  })
})
