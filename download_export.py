#!/usr/bin/env python3
"""
Automate login to ClasseViva and download the homework calendar export.
Uses Playwright instead of Selenium for better macOS compatibility.
"""

import os
import sys
import time
from pathlib import Path
from datetime import datetime
from calendar import monthrange

try:
    from dotenv import load_dotenv
    from playwright.sync_api import sync_playwright, TimeoutError as PlaywrightTimeout
except ImportError as e:
    print(f"Error: Missing required package: {e}")
    print("Install with: pip3 install playwright python-dotenv")
    print("Then run: playwright install chromium")
    sys.exit(1)

def login_to_classeviva(page, username, password):
    """Login to ClasseViva."""
    print("\n" + "=" * 60)
    print("STEP 1: Logging into ClasseViva")
    print("=" * 60)
    print(f"Navigating to ClasseViva login page...")
    
    try:
        page.goto("https://web.spaggiari.eu/home/app/default/login.php", wait_until="networkidle", timeout=30000)
        print(f"✓ Page loaded: {page.url}")
    except Exception as e:
        print(f"❌ Failed to load login page: {e}")
        return False
    
    try:
        # Find and fill username
        print("Looking for username field...")
        username_field = page.wait_for_selector("#login", timeout=10000)
        print("✓ Username field found")
        username_field.fill(username)
        print(f"✓ Username entered (length: {len(username)})")
        
        # Find and fill password
        print("Looking for password field...")
        password_field = page.locator("#password")
        if not password_field.count():
            print("❌ Password field not found!")
            return False
        print("✓ Password field found")
        password_field.fill(password)
        print(f"✓ Password entered (length: {len(password)})")
        
        # Check for CAPTCHA or other security measures
        print("Checking for CAPTCHA or security measures...")
        captcha_selectors = [
            "iframe[src*='recaptcha']",
            ".g-recaptcha",
            "#recaptcha",
            "[data-sitekey]",
            ".captcha"
        ]
        captcha_found = False
        for selector in captcha_selectors:
            try:
                if page.locator(selector).count() > 0:
                    print(f"⚠️  CAPTCHA detected: {selector}")
                    captcha_found = True
            except:
                pass
        
        if captcha_found:
            print("❌ CAPTCHA detected - manual intervention required")
            print("   Try running with --interactive flag to complete CAPTCHA manually")
            return False
        
        # Submit form - try multiple methods
        print("Looking for login button...")
        login_button = None
        
        # Try different selectors for login button
        button_selectors = [
            "input[type='submit']",
            "button[type='submit']",
            ".btn-login",
            "button:has-text('Accedi')",
            "button:has-text('Login')",
            "input[value*='Accedi']",
            "input[value*='Login']",
            "#login-button",
            ".login-button"
        ]
        
        for selector in button_selectors:
            try:
                btn = page.locator(selector).first
                if btn.count() > 0 and btn.is_visible():
                    login_button = btn
                    button_text = btn.text_content() or btn.get_attribute("value") or ""
                    print(f"✓ Login button found using selector: {selector}")
                    if button_text:
                        print(f"   Button text: '{button_text.strip()}'")
                    break
            except:
                continue
        
        if not login_button:
            print("❌ Login button not found with any selector!")
            # List all buttons on the page for debugging
            try:
                all_buttons = page.locator("button, input[type='submit'], input[type='button']").all()
                print(f"   Found {len(all_buttons)} buttons on page:")
                for i, btn in enumerate(all_buttons[:5]):
                    try:
                        text = btn.text_content() or btn.get_attribute("value") or ""
                        if text.strip():
                            print(f"     - Button {i+1}: '{text.strip()}'")
                    except:
                        pass
            except:
                pass
            return False
        
        print("Clicking login button...")
        try:
            # Try clicking with wait for navigation
            with page.expect_navigation(timeout=15000, wait_until="networkidle"):
                login_button.click()
            print("✓ Login button clicked, navigation detected")
        except Exception as nav_error:
            print(f"⚠️  Navigation not detected after click: {nav_error}")
            print("   Trying alternative: clicking and waiting...")
            login_button.click()
            time.sleep(3)  # Wait a bit longer
            try:
                page.wait_for_load_state("networkidle", timeout=10000)
            except:
                print("   Still waiting for page to load...")
        
        # Additional wait to ensure page has loaded
        time.sleep(2)
        
        # Check if login was successful
        current_url = page.url
        print(f"Current URL after login: {current_url}")
        
        if "login" in current_url.lower():
            print("⚠️  Warning: Still on login page, checking for errors...")
            
            # Check for various error message selectors
            error_found = False
            error_selectors = [
                ".alert-danger",
                ".error",
                ".msg-error",
                ".alert",
                "[role='alert']",
                ".notification-error",
                ".login-error",
                "#error",
                ".error-message"
            ]
            
            for selector in error_selectors:
                try:
                    error_elem = page.locator(selector).first
                    if error_elem.is_visible():
                        error_text = error_elem.text_content()
                        if error_text and error_text.strip():
                            print(f"❌ Login error message found ({selector}): {error_text.strip()}")
                            error_found = True
                            break
                except:
                    continue
            
            if not error_found:
                print("   (No visible error message found)")
            
            # Save page HTML and screenshot for debugging
            try:
                debug_dir = Path(__file__).parent / 'data'
                debug_dir.mkdir(exist_ok=True)
                
                # Save HTML
                html_path = debug_dir / "login_failed_page.html"
                with open(html_path, 'w', encoding='utf-8') as f:
                    f.write(page.content())
                print(f"📄 Page HTML saved to: {html_path}")
                
                # Save screenshot
                screenshot_path = debug_dir / "login_failed_screenshot.png"
                page.screenshot(path=str(screenshot_path), full_page=True)
                print(f"📸 Screenshot saved to: {screenshot_path}")
                
                # Try to get page title and some visible text
                try:
                    title = page.title()
                    print(f"   Page title: {title}")
                except:
                    pass
                
                # Check if username/password fields still have values
                try:
                    username_value = username_field.input_value()
                    password_value = password_field.input_value()
                    print(f"   Username field value: {username_value[:3]}... (length: {len(username_value)})")
                    print(f"   Password field value: {'*' * min(len(password_value), 10)}... (length: {len(password_value)})")
                except:
                    pass
                
            except Exception as e:
                print(f"   Could not save debug files: {e}")
            
            print("❌ Login appears to have failed (still on login page)")
            print("\n💡 Debugging tips:")
            print("   1. Check the saved HTML and screenshot files")
            print("   2. Try running with --interactive to see what's happening")
            print("   3. Verify your credentials are correct")
            print("   4. Check if ClasseViva requires CAPTCHA or 2FA")
            return False
        
        print("✅ Login successful!")
        return True
        
    except PlaywrightTimeout as e:
        print(f"❌ Error: Timeout waiting for login form elements: {e}")
        print(f"   Current URL: {page.url}")
        return False
    except Exception as e:
        print(f"❌ Error during login: {e}")
        import traceback
        traceback.print_exc()
        return False

def navigate_to_export(page):
    """Navigate to the calendar export page."""
    print("\n" + "=" * 60)
    print("STEP 2: Navigating to agenda page")
    print("=" * 60)
    
    try:
        # Exact URL for ClasseViva agenda
        agenda_url = "https://web.spaggiari.eu/fml/app/default/agenda_studenti.php"
        
        print(f"Loading: {agenda_url}")
        page.goto(agenda_url, wait_until="networkidle", timeout=30000)
        time.sleep(2)
        
        current_url = page.url
        print(f"Current URL after navigation: {current_url}")
        
        # Check if we're on the right page
        if "agenda" in current_url.lower():
            print("✅ Successfully navigated to agenda page")
            return True
        else:
            print(f"⚠️  Warning: URL changed to: {current_url}")
            print("   (Continuing anyway - might be redirected)")
            return True
        
    except Exception as e:
        print(f"❌ Error navigating to export: {e}")
        import traceback
        traceback.print_exc()
        return False

def download_export(page, download_dir, interactive=False):
    """Download the export file by clicking 'Scarica in Excel' button and handling the modal."""
    print("\n" + "="*60)
    print("Looking for 'Scarica in Excel' button...")
    print("="*60)
    
    try:
        # Find the "Scarica in Excel" button - it's in the top right
        print("Searching for button with text 'Scarica in Excel'...")
        
        button_found = False
        export_button = None
        
        # Method 1: Find by exact text
        try:
            export_button = page.locator("text=Scarica in Excel").first
            if export_button.is_visible():
                print("✓ Found 'Scarica in Excel' button using text search")
                button_found = True
        except:
            pass
        
        # Method 2: Find by title attribute
        if not button_found:
            try:
                export_button = page.locator("[title='Scarica in Excel']").first
                if export_button.is_visible():
                    print("✓ Found 'Scarica in Excel' button using title attribute")
                    button_found = True
            except:
                pass
        
        # Method 3: Find any button/link containing "Scarica" or "Excel"
        if not button_found:
            try:
                all_buttons = page.locator("button, a").all()
                for elem in all_buttons:
                    try:
                        text = elem.text_content() or ""
                        title = elem.get_attribute("title") or ""
                        if "Scarica" in text or "Scarica" in title or "Excel" in text or "Excel" in title:
                            if elem.is_visible():
                                print(f"✓ Found potential export button: text='{text}', title='{title}'")
                                export_button = elem
                                button_found = True
                                break
                    except:
                        continue
            except Exception as e:
                print(f"  Error in fallback search: {e}")
        
        if not button_found or export_button is None:
            print("❌ Could not find 'Scarica in Excel' button")
            print(f"   Current URL: {page.url}")
            print("   Taking screenshot for debugging...")
            screenshot_path = download_dir / "classeviva_screenshot.png"
            try:
                page.screenshot(path=str(screenshot_path), full_page=True)
                print(f"📸 Screenshot saved to: {screenshot_path}")
            except Exception as e:
                print(f"   Could not save screenshot: {e}")
            
            # Try to get page title and some text for debugging
            try:
                title = page.title()
                print(f"   Page title: {title}")
            except:
                pass
            
            if interactive:
                print("\nBrowser is open - please manually click the 'Scarica in Excel' button.")
                print("Press Enter when the modal appears...")
                input()
            else:
                print("   Try running with --interactive to see what's on the page")
                return False
        
        # Set up download listener and new window listener
        download_promise = None
        print("\nSetting up download listener and clicking export button...")
        
        # Set up network request monitoring
        network_requests = []
        def handle_request(request):
            url = request.url
            if any(word in url.lower() for word in ["export", "scarica", "excel", "xls", "download", "agenda"]):
                network_requests.append(url)
                print(f"   📡 Network request detected: {url[:150]}")
        
        page.on("request", handle_request)
        
        # Inspect the button more thoroughly before clicking
        print("Inspecting button properties...")
        try:
            button_tag = export_button.evaluate("el => el.tagName.toLowerCase()")
            button_href = export_button.get_attribute("href") or ""
            button_onclick = export_button.evaluate("el => el.getAttribute('onclick')") or ""
            button_type = export_button.get_attribute("type") or ""
            button_class = export_button.get_attribute("class") or ""
            button_id = export_button.get_attribute("id") or ""
            
            print(f"   Button tag: {button_tag}")
            print(f"   Button class: {button_class[:100]}")
            print(f"   Button ID: {button_id}")
            if button_href:
                print(f"   Button href: {button_href[:150]}")
            if button_onclick:
                print(f"   Button onclick: {button_onclick[:200]}")
            if button_type:
                print(f"   Button type: {button_type}")
            
            # Check if button has data attributes
            try:
                data_attrs = export_button.evaluate("""
                    el => {
                        const attrs = {};
                        for (let attr of el.attributes) {
                            if (attr.name.startsWith('data-')) {
                                attrs[attr.name] = attr.value;
                            }
                        }
                        return attrs;
                    }
                """)
                if data_attrs:
                    print(f"   Button data attributes: {data_attrs}")
            except:
                pass
        except Exception as e:
            print(f"   Could not inspect button: {e}")
        
        # Try to wait for download, but also handle modal and new windows
        try:
            context = page.context
            new_page = None
            
            # Set up listener for new windows/popups (will be checked after click)
            def check_new_windows():
                nonlocal new_page
                try:
                    pages = context.pages
                    if len(pages) > 1:
                        # A new page/window opened
                        new_page = pages[-1]  # Get the most recent page
                        print(f"   🪟 New window/page detected: {new_page.url}")
                        return True
                except:
                    pass
                return False
            
            # Variables to track modal state (accessible outside with block)
            modal_found = False
            modal_element = None
            
            with page.expect_download(timeout=30000) as download_info:
                # Click the button to open the modal or trigger download
                print("Clicking 'Scarica in Excel' button...")
                try:
                    # Try normal click first
                    export_button.click()
                except Exception as click_error:
                    print(f"⚠️  Error with normal click: {click_error}")
                    print("   Trying alternative click methods...")
                    try:
                        # Try JavaScript click
                        page.evaluate("element => element.click()", export_button)
                    except:
                        try:
                            # Try force click
                            export_button.click(force=True)
                        except Exception as e2:
                            print(f"❌ All click methods failed: {e2}")
                            raise
                
                # Wait a bit for modal to appear or download to start
                print("Waiting for modal to appear or download to start...")
                
                # Check for new windows/popups that might have opened
                time.sleep(1)  # Give it a moment for new window to open
                if check_new_windows():
                    print("   ⚠️  New window detected - checking if download is there...")
                    try:
                        # Wait for the new page to load
                        new_page.wait_for_load_state("networkidle", timeout=5000)
                        new_page_url = new_page.url
                        print(f"   New page URL: {new_page_url}")
                        
                        # Check if it's a download URL
                        if any(ext in new_page_url.lower() for ext in [".xls", ".xlsx", ".csv"]):
                            print("   ✓ New page appears to be a download URL!")
                            # The download should be captured by the download listener
                        else:
                            # Maybe the modal is in the new window
                            print("   Checking new window for modal...")
                            # We could switch to the new page, but let's first check the main page
                    except Exception as e:
                        print(f"   Error checking new window: {e}")
                
                # Wait for any network activity or page changes
                try:
                    # Wait a short time for immediate responses
                    page.wait_for_load_state("networkidle", timeout=3000)
                    print("   Page reached network idle state")
                except:
                    # Network idle timeout is OK - page might still be loading or download might start
                    pass
                
                # Check network requests that were made
                if network_requests:
                    print(f"   Found {len(network_requests)} relevant network requests")
                    for req in network_requests[:3]:
                        print(f"     - {req[:150]}")
                
                time.sleep(1)  # Additional small wait
                
                # Take a screenshot to see what's on the page
                try:
                    debug_screenshot = download_dir / "after_click_screenshot.png"
                    page.screenshot(path=str(debug_screenshot), full_page=True)
                    print(f"📸 Debug screenshot saved: {debug_screenshot}")
                except:
                    pass
                
                # Now handle the modal dialog - wait a bit for it to appear
                print("Looking for export modal dialog...")
                print("   (Waiting up to 10 seconds for modal to appear after button click)...")
                
                # Prioritize .ui-dialog since we know that's what the page uses
                ui_dialog_selectors = [
                    ".ui-dialog",
                    ".ui-dialog:visible",
                    "div.ui-dialog",
                    "[class*='ui-dialog']:visible"
                ]
                
                # Try ui-dialog first with explicit wait
                print("   Checking for .ui-dialog element...")
                for selector in ui_dialog_selectors:
                    try:
                        modal = page.locator(selector).first
                        # Wait for it to appear and become visible
                        try:
                            modal.wait_for(state="visible", timeout=5000)
                            if modal.is_visible():
                                print(f"✓ Modal dialog found using selector: {selector}")
                                modal_found = True
                                modal_element = modal
                                break
                        except:
                            # Try waiting for attached first, then visible
                            try:
                                modal.wait_for(state="attached", timeout=2000)
                                # Give it a moment to become visible
                                time.sleep(0.5)
                                if modal.is_visible():
                                    print(f"✓ Modal dialog found using selector: {selector}")
                                    modal_found = True
                                    modal_element = modal
                                    break
                            except:
                                continue
                    except:
                        continue
                
                # If ui-dialog not found, try other selectors
                if not modal_found:
                    modal_selectors = [
                        ".modal",
                        ".modal-dialog",
                        ".dialog",
                        "[role='dialog']",
                        "#exportModal",
                        ".popup",
                        ".overlay",
                        "[class*='modal']",
                        "[class*='dialog']",
                        "[id*='modal']",
                        "[id*='dialog']",
                        ".fancybox-overlay",
                        ".fancybox-wrap",
                        "[style*='display: block'][style*='z-index']"
                    ]
                    
                    # Wait up to 5 seconds for modal to appear, checking every 500ms
                    for attempt in range(10):  # 10 attempts * 500ms = 5 seconds
                        for selector in modal_selectors:
                            try:
                                modal = page.locator(selector).first
                                if modal.count() > 0:
                                    # Check if it's visible
                                    try:
                                        if modal.is_visible():
                                            print(f"✓ Modal dialog found using selector: {selector}")
                                            modal_found = True
                                            modal_element = modal
                                            break
                                    except:
                                        # Try next selector
                                        continue
                            except:
                                continue
                        
                        if modal_found:
                            break
                        
                        # Wait a bit before next check
                        time.sleep(0.5)
                
                if not modal_found:
                    print("⚠️  Modal dialog not found with standard selectors")
                    print("   Checking page for any visible dialogs/modals...")
                    
                    # Check for iframes (modals might be in iframes)
                    try:
                        iframes = page.locator("iframe").all()
                        if iframes:
                            print(f"   Found {len(iframes)} iframe(s) - modal might be inside")
                            for i, iframe in enumerate(iframes):
                                try:
                                    iframe_src = iframe.get_attribute("src") or ""
                                    print(f"     - Iframe {i+1}: {iframe_src[:100]}")
                                except:
                                    pass
                    except:
                        pass
                    
                    # Check for any elements that might be modals (high z-index, overlay, etc.)
                    try:
                        # Look for elements with modal-like classes or attributes
                        potential_modals = page.locator("[class*='popup'], [class*='overlay'], [style*='z-index'], [style*='display: block'], [class*='dialog'], [class*='modal']").all()
                        if potential_modals:
                            print(f"   Found {len(potential_modals)} potential modal elements")
                            # Check if any are actually visible or might be modals
                            for i, elem in enumerate(potential_modals[:10]):
                                try:
                                    is_visible = elem.is_visible()
                                    text = elem.text_content() or ""
                                    classes = elem.get_attribute("class") or ""
                                    tag = elem.evaluate("el => el.tagName.toLowerCase()")
                                    z_index = elem.evaluate("el => window.getComputedStyle(el).zIndex") or ""
                                    display = elem.evaluate("el => window.getComputedStyle(el).display") or ""
                                    
                                    print(f"     - Element {i+1} ({tag}): visible={is_visible}, class='{classes[:50]}', z-index={z_index}, display={display}")
                                    if text.strip():
                                        print(f"       text='{text[:100]}'")
                                    
                                    # If it looks like a modal (high z-index, block display, or has modal-like content)
                                    looks_like_modal = False
                                    if is_visible:
                                        # Check z-index
                                        if z_index and z_index.isdigit() and int(z_index) > 1000:
                                            looks_like_modal = True
                                        # Check display
                                        if display == "block":
                                            looks_like_modal = True
                                        # Check for modal-like keywords
                                        if any(word in (classes + text).lower() for word in ["modal", "dialog", "popup", "overlay", "export", "scarica", "excel", "date", "dal", "al", "conferma"]):
                                            looks_like_modal = True
                                    
                                    if looks_like_modal:
                                        print(f"       → This looks like a modal! Trying to use it...")
                                        # Check if it contains date input fields or confirm buttons
                                        try:
                                            date_fields = elem.locator("input[name*='dal'], input[name*='al'], input[id*='dal'], input[id*='al']").all()
                                            confirm_buttons = elem.locator("button, a, input[type='button']").all()
                                            if date_fields or confirm_buttons:
                                                print(f"       → Found {len(date_fields)} date fields and {len(confirm_buttons)} buttons inside!")
                                                modal_found = True
                                                modal_element = elem
                                                break
                                        except:
                                            # Even if we can't find fields, if it looks like a modal, use it
                                            modal_found = True
                                            modal_element = elem
                                            break
                                except Exception as e:
                                    # Continue checking other elements
                                    continue
                    except Exception as e:
                        print(f"   Error checking potential modals: {e}")
                        pass
                    
                    # Check if download started directly (no modal)
                    print("   Checking if download started directly (no modal needed)...")
                    print("   (If download started, it will be captured by the download listener)")
                    # Don't wait here - let the download listener handle it
                else:
                    print("   Modal found, proceeding with form filling...")
                
                # Only proceed with date filling if modal was found
                if modal_found:
                    print("Proceeding with date range setup...")
                else:
                    print("⚠️  No modal found - trying alternative approaches...")
                    
                    # Try to find if the button has a direct download URL
                    try:
                        button_href = export_button.get_attribute("href")
                        if button_href and (".xls" in button_href.lower() or "export" in button_href.lower() or "download" in button_href.lower()):
                            print(f"   Button has direct download URL: {button_href[:100]}")
                            print("   Download should start automatically...")
                    except:
                        pass
                    
                    # Check if any new download links appeared on the page
                    try:
                        download_links = page.locator("a[href*='.xls'], a[href*='export'], a[download], [onclick*='download'], [onclick*='export']").all()
                        if download_links:
                            print(f"   Found {len(download_links)} potential download links after button click")
                            for i, link in enumerate(download_links[:3]):
                                try:
                                    href = link.get_attribute("href") or ""
                                    onclick = link.get_attribute("onclick") or ""
                                    text = link.text_content() or ""
                                    if href or onclick:
                                        print(f"     - Link {i+1}: {text[:30]} -> {href[:80] or onclick[:80]}")
                                        # If it looks like an export link, try clicking it
                                        if any(word in (href + onclick + text).lower() for word in ["export", "scarica", "excel", "xls", "download"]):
                                            print(f"       → Trying to click this download link...")
                                            try:
                                                link.click()
                                                time.sleep(2)
                                            except:
                                                pass
                                except:
                                    pass
                    except:
                        pass
                    
                    # Wait a bit more and check for modal again (maybe it appears with delay)
                    print("   Waiting a bit more and re-checking for modal...")
                    time.sleep(3)
                    
                    # Re-check for modal one more time
                    for selector in [".modal", ".modal-dialog", "[role='dialog']", "[class*='modal']", "[class*='dialog']"]:
                        try:
                            modal = page.locator(selector).first
                            if modal.count() > 0 and modal.is_visible():
                                print(f"   ✓ Modal found on second check using: {selector}")
                                modal_found = True
                                break
                        except:
                            continue
                    
                    if not modal_found:
                        print("   No modal found - trying alternative download methods...")
                        
                        # First, try navigating directly to any download URLs we found in network requests
                        if network_requests:
                            print(f"   Trying to navigate to download URLs from network requests...")
                            for req_url in network_requests:
                                if any(ext in req_url.lower() for ext in [".xls", ".xlsx", ".csv"]) or "download" in req_url.lower():
                                    print(f"   → Attempting to navigate to: {req_url[:150]}")
                                    try:
                                        # Navigate to the URL - this should trigger a download
                                        page.goto(req_url, wait_until="networkidle", timeout=10000)
                                        time.sleep(2)
                                        print("   ✓ Navigated to download URL")
                                        break
                                    except Exception as e:
                                        print(f"   Could not navigate to URL: {e}")
                                        continue
                        
                        # Try JavaScript approach
                        try:
                            # Check if button has onclick handler we can call
                            try:
                                button_onclick = export_button.evaluate("el => el.getAttribute('onclick')") or ""
                                if button_onclick:
                                    print(f"   Button has onclick: {button_onclick[:100]}")
                                    # Try to execute it
                                    try:
                                        page.evaluate(f"() => {{ {button_onclick} }}")
                                        print("   ✓ Executed button's onclick handler")
                                        time.sleep(2)
                                    except Exception as e:
                                        print(f"   Could not execute onclick: {e}")
                            except:
                                pass
                            
                            # Try clicking the button again with JavaScript
                            try:
                                export_button.evaluate("el => el.click()")
                                print("   ✓ Clicked button again via JavaScript")
                                time.sleep(2)
                            except:
                                pass
                                
                        except Exception as e:
                            print(f"   JavaScript approach failed: {e}")
                        
                        # Try constructing a download URL based on the current page URL
                        try:
                            current_url = page.url
                            if "agenda_studenti.php" in current_url:
                                # Try common export URL patterns
                                base_url = current_url.split("?")[0] if "?" in current_url else current_url
                                export_urls = [
                                    f"{base_url}?action=export&format=xls",
                                    f"{base_url}?export=xls",
                                    f"{base_url}?scarica=excel",
                                    f"{base_url.replace('agenda_studenti.php', 'export_agenda.php')}",
                                ]
                                
                                print("   Trying common export URL patterns...")
                                for export_url in export_urls:
                                    try:
                                        print(f"   → Trying: {export_url}")
                                        # Navigate to the URL - download should be captured by listener
                                        response = page.goto(export_url, wait_until="networkidle", timeout=10000)
                                        time.sleep(2)
                                        
                                        # Check response headers for content type
                                        try:
                                            content_type = response.headers.get("content-type", "").lower()
                                            content_disposition = response.headers.get("content-disposition", "").lower()
                                            
                                            print(f"   Response Content-Type: {content_type}")
                                            if content_disposition:
                                                print(f"   Content-Disposition: {content_disposition}")
                                            
                                            # Check if it's a download
                                            if any(ct in content_type for ct in ["application/vnd.ms-excel", "application/excel", "application/octet-stream", "application/x-msexcel"]):
                                                print(f"   ✓ Response is Excel file - download should be captured!")
                                                time.sleep(3)  # Give download time to start
                                                break
                                            elif "attachment" in content_disposition or ".xls" in content_disposition:
                                                print(f"   ✓ Content-Disposition indicates download!")
                                                time.sleep(3)
                                                break
                                        except Exception as header_error:
                                            print(f"   Could not check headers: {header_error}")
                                        
                                        # Check if download started or if we got redirected
                                        if page.url != export_url:
                                            print(f"   Redirected to: {page.url}")
                                        
                                        # Check what's on the page - might be a form or direct download
                                        try:
                                            page_title = page.title()
                                            page_content = page.content()
                                            
                                            # Check if it's HTML (form) or binary (download)
                                            # Excel files typically start with specific bytes or are not valid HTML
                                            is_binary = False
                                            if len(page_content) < 50000:  # Small file might be download
                                                # Check if it looks like binary data
                                                if not page_content.strip().startswith("<") or "<?xml" not in page_content[:100]:
                                                    # Check for Excel file signatures
                                                    content_start = page_content[:20] if len(page_content) >= 20 else page_content
                                                    # Excel files might start with PK (ZIP signature) or other binary data
                                                    if any(ord(c) < 32 and c not in '\t\n\r' for c in content_start[:10]):
                                                        is_binary = True
                                                        print(f"   ✓ Page appears to be binary/download (size: {len(page_content)} bytes)")
                                                        print(f"   Download should be captured by download listener")
                                                        time.sleep(2)
                                                        break
                                            
                                            if not is_binary:
                                                print(f"   Page is HTML - checking for forms...")
                                                # Look for form fields
                                                forms = page.locator("form").all()
                                                if forms:
                                                    print(f"   Found {len(forms)} form(s) on the page")
                                                    # Look for date fields and submit buttons
                                                    date_fields = page.locator("input[name*='dal'], input[name*='al'], input[id*='dal'], input[id*='al']").all()
                                                    submit_buttons = page.locator("input[type='submit'], button[type='submit'], button:has-text('Conferma'), button:has-text('Scarica')").all()
                                                    
                                                    if date_fields or submit_buttons:
                                                        print(f"   Found {len(date_fields)} date fields and {len(submit_buttons)} submit buttons")
                                                        # Fill date fields if found
                                                        if date_fields:
                                                            today = datetime.now()
                                                            year = today.year
                                                            month = today.month
                                                            date_from = datetime(year, month, 1)
                                                            last_day = monthrange(year, month)[1]
                                                            date_to = datetime(year, month, last_day)
                                                            date_from_str = date_from.strftime("%d-%m-%Y")
                                                            date_to_str = date_to.strftime("%d-%m-%Y")
                                                            
                                                            for field in date_fields:
                                                                try:
                                                                    name = field.get_attribute("name") or ""
                                                                    if "dal" in name.lower():
                                                                        field.fill(date_from_str)
                                                                        print(f"   ✓ Filled 'dal' field: {date_from_str}")
                                                                    elif "al" in name.lower():
                                                                        field.fill(date_to_str)
                                                                        print(f"   ✓ Filled 'al' field: {date_to_str}")
                                                                except:
                                                                    pass
                                                        
                                                        # Click submit button
                                                        if submit_buttons:
                                                            try:
                                                                submit_buttons[0].click()
                                                                print(f"   ✓ Clicked submit button - download should start...")
                                                                time.sleep(3)
                                                                break
                                                            except Exception as e:
                                                                print(f"   Could not click submit: {e}")
                                                else:
                                                    # No form - maybe it's a direct download page
                                                    print(f"   No form found - page might trigger download automatically")
                                                    time.sleep(2)
                                        except Exception as e:
                                            print(f"   Error checking page content: {e}")
                                            
                                    except Exception as e:
                                        print(f"   Error navigating to {export_url}: {e}")
                                        continue
                        except Exception as e:
                            print(f"   URL pattern approach failed: {e}")
                        
                        print("   Waiting for download to start (up to 30 seconds)...")
                        print("   (The download listener will capture it when it starts)")
                    # Don't return early - let the download listener wait for the full timeout
                    # The download might start after a delay, so we need to wait within the 'with' block
                    # Just continue - the download listener will either capture it or timeout
            
            # Fill in date range to full current month (only if modal was found)
            if not modal_found:
                print("⚠️  Skipping date filling - modal not found")
                # This code shouldn't be reached if modal wasn't found due to return above
            else:
                print("Setting date range to full current month...")
            
            try:
                # Get current date and set range to full current month
                today = datetime.now()
                year = today.year
                month = today.month
                
                # First day of current month
                date_from = datetime(year, month, 1)
                # Last day of current month
                last_day = monthrange(year, month)[1]
                date_to = datetime(year, month, last_day)
                
                # Format as DD-MM-YYYY (Italian format)
                date_from_str = date_from.strftime("%d-%m-%Y")
                date_to_str = date_to.strftime("%d-%m-%Y")
                
                print(f"  Setting date range: {date_from_str} to {date_to_str}")
                
                # Try to find and fill date input fields
                # Use modal_element if we found it, otherwise search the whole page
                search_context = modal_element if modal_element else page
                
                date_selectors = [
                    ("input[name*='dal']", "From date field"),
                    ("input[name*='al']", "To date field"),
                    ("input[id*='dal']", "From date field by ID"),
                    ("input[id*='al']", "To date field by ID"),
                ]
                
                print(f"  Searching for date fields in {'modal' if modal_element else 'page'}...")
                for selector, description in date_selectors:
                    try:
                        field = search_context.locator(selector).first
                        if field.count() > 0 and field.is_visible():
                            if "dal" in selector.lower():
                                field.fill(date_from_str)
                                print(f"  ✓ Set 'dal' date: {date_from_str}")
                            elif "al" in selector.lower():
                                field.fill(date_to_str)
                                print(f"  ✓ Set 'al' date: {date_to_str}")
                    except Exception as e:
                        print(f"  Could not fill field {selector}: {e}")
                        continue
                
                # Make sure xls format is selected (it should be by default)
                print("  Ensuring xls format is selected...")
                try:
                    xls_radio = search_context.locator("input[value*='xls'], input[type='radio'][value*='office']").first
                    if xls_radio.count() > 0 and not xls_radio.is_checked():
                        xls_radio.check()
                        print("  ✓ Selected xls format")
                except:
                    print("  (xls format should already be selected)")
                
            except Exception as e:
                print(f"  Note: Could not set dates (using defaults): {e}")
            
            # Click "Conferma" (Confirm) button
            print("\nLooking for 'Conferma' (Confirm) button to download...")
            
            # Use modal_element if we found it, otherwise search the whole page
            search_context = modal_element if modal_element else page
            
            confirm_button = None
            confirm_selectors = [
                ("button:has-text('Conferma')", "Button with 'Conferma' text"),
                ("a:has-text('Conferma')", "Link with 'Conferma' text"),
                ("input[value='Conferma']", "Input with 'Conferma' value"),
                ("button.confirm, .btn-confirm, #btn-confirm", "Button with confirm class/id"),
                ("button:has-text('OK')", "Button with 'OK' text"),
                ("button:has-text('Download')", "Button with 'Download' text"),
                ("button:has-text('Scarica')", "Button with 'Scarica' text"),
                ("[onclick*='download']", "Element with download onclick"),
                ("[onclick*='export']", "Element with export onclick"),
            ]
            
            # Also try to find any clickable element in the modal
            for selector, description in confirm_selectors:
                try:
                    btn = search_context.locator(selector).first
                    if btn.count() > 0:
                        # Check if visible or in a modal
                        try:
                            if btn.is_visible():
                                confirm_button = btn
                                print(f"✓ Found 'Conferma' button: {description}")
                                break
                        except:
                            # Might be in modal, try anyway
                            confirm_button = btn
                            print(f"✓ Found potential button: {description}")
                            break
                except:
                    continue
            
            # If not found, list ALL clickable elements for debugging
            if confirm_button is None:
                print("⚠️  Could not find 'Conferma' button with standard selectors")
                print("   Searching for all clickable elements...")
                
                try:
                    # Find all buttons, links, and clickable elements
                    all_clickable = search_context.locator("button, a, input[type='button'], input[type='submit'], [onclick], [role='button']").all()
                    print(f"   Found {len(all_clickable)} clickable elements:")
                    
                    visible_count = 0
                    for i, elem in enumerate(all_clickable[:15]):  # Show first 15
                        try:
                            if elem.is_visible():
                                visible_count += 1
                                text = elem.text_content() or elem.get_attribute("value") or elem.get_attribute("title") or ""
                                tag = elem.evaluate("el => el.tagName.toLowerCase()")
                                classes = elem.get_attribute("class") or ""
                                if text.strip() or "conferma" in text.lower() or "scarica" in text.lower() or "download" in text.lower():
                                    print(f"     - {tag} {i+1}: '{text.strip()}' (class: {classes[:50]})")
                                    # If it looks like a confirm button, use it
                                    if any(word in text.lower() for word in ["conferma", "ok", "scarica", "download", "export"]):
                                        confirm_button = elem
                                        print(f"       → Using this as confirm button!")
                                        break
                        except:
                            pass
                    
                    print(f"   ({visible_count} visible out of {len(all_clickable)} total)")
                    
                except Exception as e:
                    print(f"   Could not list elements: {e}")
            
            # Try clicking the button if found
            if confirm_button:
                try:
                    print(f"Clicking confirm button...")
                    # Try multiple click methods
                    try:
                        confirm_button.click()
                    except:
                        try:
                            confirm_button.click(force=True)
                        except:
                            try:
                                # Use JavaScript click as fallback
                                page.evaluate("element => element.click()", confirm_button)
                            except:
                                # Try scrolling into view first
                                confirm_button.scroll_into_view_if_needed()
                                confirm_button.click()
                    
                    print("✓ Clicked confirm button - download should start...")
                    time.sleep(3)  # Wait a bit longer for download to start
                except Exception as e:
                    print(f"⚠️  Error clicking button: {e}")
                    import traceback
                    traceback.print_exc()
            else:
                print("❌ Could not find 'Conferma' button automatically")
                print(f"   Current URL: {page.url}")
                
                # Save debug info
                screenshot_path = download_dir / "classeviva_modal_screenshot.png"
                try:
                    page.screenshot(path=str(screenshot_path), full_page=True)
                    print(f"📸 Screenshot saved to: {screenshot_path}")
                except Exception as e:
                    print(f"   Could not save screenshot: {e}")
                
                # Save page HTML
                try:
                    html_path = download_dir / "classeviva_modal_page.html"
                    with open(html_path, 'w', encoding='utf-8') as f:
                        f.write(page.content())
                    print(f"📄 Page HTML saved to: {html_path}")
                except:
                    pass
                
                if interactive:
                    print("\nBrowser is open - please manually click 'Conferma' in the modal.")
                    print("Press Enter when download starts...")
                    input()
                else:
                    print("   Try running with --interactive to see what's on the page")
                    print("   Or check the saved screenshot and HTML files")
                    # Don't return False yet - maybe download started directly
                    print("   Waiting to see if download started directly...")
                    time.sleep(5)  # Wait longer to see if download happens
                    
                    # Also try pressing Enter in case there's a focused button
                    try:
                        page.keyboard.press("Enter")
                        print("   Tried pressing Enter key...")
                        time.sleep(2)
                    except:
                        pass
            
            # Wait for download to start
            time.sleep(2)
            
            # Get the download (after 'with' block ends, download_info.value is still accessible)
            print("Waiting for download to complete...")
            download = download_info.value
            print(f"✓ Download started: {download.suggested_filename}")
            
            # Save the file
            timestamp = datetime.now().strftime("%Y%m%d")
            filename = download_dir / f"export_{timestamp}.xls"
            
            print(f"Saving download to: {filename}")
            if filename.exists():
                print(f"   (Removing existing file: {filename})")
                filename.unlink()
            
            download.save_as(filename)
            if filename.exists():
                file_size = filename.stat().st_size
                print(f"✅ Download saved successfully!")
                print(f"   File: {filename}")
                print(f"   Size: {file_size} bytes")
                return filename
            else:
                print(f"❌ File was not saved (file does not exist after save)")
                return False
                
        except PlaywrightTimeout:
            print("❌ Timeout waiting for download to start")
        print(f"   Current URL: {page.url}")
        screenshot_path = download_dir / "classeviva_timeout_screenshot.png"
        try:
            page.screenshot(path=str(screenshot_path), full_page=True)
            print(f"📸 Screenshot saved to: {screenshot_path}")
        except:
            pass
        return False
    except Exception as e:
        print(f"❌ Error downloading export: {e}")
        print(f"   Current URL: {page.url}")
        import traceback
        traceback.print_exc()
        return False

def cleanup_old_exports(data_dir, keep_days=7):
    """Remove old export files, keeping only the most recent ones."""
    try:
        from datetime import timedelta
        
        # Find all export files
        export_files = list(data_dir.glob("export_*.xls*"))
        
        if len(export_files) <= 1:
            return  # No cleanup needed
        
        # Sort by modification time (newest first)
        export_files.sort(key=lambda f: f.stat().st_mtime, reverse=True)
        
        # Calculate cutoff date
        cutoff_date = datetime.now() - timedelta(days=keep_days)
        
        deleted_count = 0
        for file in export_files:
            file_mtime = datetime.fromtimestamp(file.stat().st_mtime)
            
            # Keep files newer than cutoff OR keep the most recent file regardless
            if file_mtime < cutoff_date and file != export_files[0]:
                try:
                    file.unlink()
                    deleted_count += 1
                    print(f"  🗑️  Deleted old export: {file.name}")
                except Exception as e:
                    print(f"  ⚠️  Could not delete {file.name}: {e}")
        
        if deleted_count > 0:
            print(f"✅ Cleaned up {deleted_count} old export file(s)")
        
    except Exception as e:
        print(f"⚠️  Error during cleanup: {e}")

def main():
    # Check if environment variables are already set (from shell/Justfile)
    username = os.getenv('CLASSEVIVA_USERNAME')
    password = os.getenv('CLASSEVIVA_PASSWORD')
    
    # Only try to load .env if credentials aren't already set
    if not username or not password:
        env_path = Path(__file__).parent / '.env'
        try:
            if env_path.exists():
                load_dotenv(env_path)
            else:
                load_dotenv()
                # Try loading from parent directory too
                parent_env = Path(__file__).parent.parent / '.env'
                if parent_env.exists():
                    load_dotenv(parent_env)
        except PermissionError:
            # Permission error is OK - env vars may be set via shell/Justfile
            pass
        except Exception as e:
            # Other errors are also OK - we'll check for env vars below
            print(f"Note: Could not load .env file: {e}")
        
        # Re-check after attempting to load
        username = os.getenv('CLASSEVIVA_USERNAME')
        password = os.getenv('CLASSEVIVA_PASSWORD')
    
    if not username or not password:
        print("Error: CLASSEVIVA_USERNAME and CLASSEVIVA_PASSWORD must be set")
        print("\nSet credentials using one of these methods:")
        print("1. Create a .env file in the project root:")
        print("   CLASSEVIVA_USERNAME=your_username")
        print("   CLASSEVIVA_PASSWORD=your_password")
        print("\n2. Export environment variables:")
        print("   export CLASSEVIVA_USERNAME=your_username")
        print("   export CLASSEVIVA_PASSWORD=your_password")
        print("\n3. Pass inline when running commands:")
        print("   CLASSEVIVA_USERNAME=user CLASSEVIVA_PASSWORD=pass just all")
        print("\nExample .env file location:", env_path)
        sys.exit(1)
    
    # Setup download directory
    data_dir = Path(__file__).parent / 'data'
    data_dir.mkdir(exist_ok=True)
    
    # Check for interactive mode
    interactive = '--interactive' in sys.argv or '-i' in sys.argv
    headless = not interactive
    
    # Setup Playwright
    print("=" * 60)
    print("ClasseViva Export Downloader")
    print("=" * 60)
    print(f"Username: {username[:3]}... (length: {len(username)})")
    print(f"Password: {'*' * min(len(password), 10)}... (length: {len(password)})")
    print(f"Download directory: {data_dir}")
    print("")
    print("Setting up browser...")
    if interactive:
        print("Running in INTERACTIVE mode (browser will be visible)")
    else:
        print("Running in HEADLESS mode (browser hidden)")
    
    with sync_playwright() as p:
        # Launch browser
        browser = p.chromium.launch(headless=headless)
        context = browser.new_context(
            accept_downloads=True,
            viewport={'width': 1920, 'height': 1080}
        )
        page = context.new_page()
        
        try:
            # Login
            if not login_to_classeviva(page, username, password):
                print("\n" + "=" * 60)
                print("❌ FAILED: Login unsuccessful")
                print("=" * 60)
                print("Please check your credentials:")
                print(f"  Username: {username[:3]}... (length: {len(username)})")
                print(f"  Password: {'*' * len(password)} (length: {len(password)})")
                if interactive:
                    print("\nBrowser will stay open for 30 seconds for debugging...")
                    time.sleep(30)
                browser.close()
                sys.exit(1)
            
            # Navigate to export page
            if not navigate_to_export(page):
                print("\n" + "=" * 60)
                print("❌ FAILED: Could not navigate to export page")
                print("=" * 60)
                if not interactive:
                    print("Try running with --interactive flag to see what's happening:")
                    print("  python3 download_export.py --interactive")
                browser.close()
                sys.exit(1)
            
            # Download export
            print("\n" + "=" * 60)
            print("STEP 3: Downloading export file")
            print("=" * 60)
            downloaded_file = download_export(page, data_dir, interactive=interactive)
            
            if downloaded_file:
                # Clean up old export files (keep only last 7 days)
                cleanup_old_exports(data_dir, keep_days=7)
                print("\n" + "=" * 60)
                print(f"✅ SUCCESS: Export saved to: {downloaded_file}")
                print("=" * 60)
                browser.close()
                return downloaded_file
            else:
                print("\n" + "=" * 60)
                print("❌ FAILED: Download did not complete successfully")
                print("=" * 60)
                if interactive:
                    print("Browser will stay open for 30 seconds for inspection...")
                    time.sleep(30)
                browser.close()
                sys.exit(1)  # Exit with error code
        
        except KeyboardInterrupt:
            print("\n\n" + "=" * 60)
            print("❌ Interrupted by user")
            print("=" * 60)
            browser.close()
            sys.exit(1)
        except Exception as e:
            print("\n" + "=" * 60)
            print(f"❌ UNEXPECTED ERROR: {e}")
            print("=" * 60)
            import traceback
            traceback.print_exc()
            browser.close()
            sys.exit(1)

if __name__ == '__main__':
    main()

