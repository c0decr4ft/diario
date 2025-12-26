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
        
        # Submit form
        print("Looking for login button...")
        login_button = page.locator("input[type='submit'], button[type='submit'], .btn-login").first
        if not login_button.count():
            print("❌ Login button not found!")
            return False
        print("✓ Login button found")
        print("Clicking login button...")
        login_button.click()
        
        # Wait for navigation after login
        print("Waiting for page to load after login...")
        page.wait_for_load_state("networkidle", timeout=15000)
        time.sleep(2)
        
        # Check if login was successful
        current_url = page.url
        print(f"Current URL after login: {current_url}")
        
        if "login" in current_url.lower():
            print("⚠️  Warning: Still on login page, checking for errors...")
            try:
                error_msg = page.locator(".alert-danger, .error, .msg-error").first
                if error_msg.is_visible():
                    error_text = error_msg.text_content()
                    print(f"❌ Login error message: {error_text}")
                    return False
            except:
                print("   (No visible error message found)")
            print("❌ Login appears to have failed (still on login page)")
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
        
        # Set up download listener
        download_promise = None
        with page.expect_download() as download_info:
            # Click the button to open the modal
            print("\nClicking 'Scarica in Excel' button to open modal...")
            export_button.click()
            time.sleep(2)  # Wait for modal to appear
            
            # Now handle the modal dialog
            print("Looking for export modal dialog...")
            
            # Wait for modal to appear
            try:
                # Look for modal - common selectors
                modal = page.locator(".modal, .dialog, [role='dialog'], .ui-dialog, #exportModal").first
                modal.wait_for(state="visible", timeout=5000)
                print("✓ Modal dialog opened")
            except:
                print("⚠ Modal did not appear - trying to continue anyway...")
            
            # Fill in date range to full current month
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
                date_selectors = [
                    ("input[name*='dal']", "From date field"),
                    ("input[name*='al']", "To date field"),
                    ("input[id*='dal']", "From date field by ID"),
                    ("input[id*='al']", "To date field by ID"),
                ]
                
                for selector, description in date_selectors:
                    try:
                        field = page.locator(selector).first
                        if field.is_visible():
                            if "dal" in selector.lower():
                                field.fill(date_from_str)
                                print(f"  ✓ Set 'dal' date: {date_from_str}")
                            elif "al" in selector.lower():
                                field.fill(date_to_str)
                                print(f"  ✓ Set 'al' date: {date_to_str}")
                    except:
                        continue
                
                # Make sure xls format is selected (it should be by default)
                print("  Ensuring xls format is selected...")
                try:
                    xls_radio = page.locator("input[value*='xls'], input[type='radio'][value*='office']").first
                    if not xls_radio.is_checked():
                        xls_radio.check()
                        print("  ✓ Selected xls format")
                except:
                    print("  (xls format should already be selected)")
                
            except Exception as e:
                print(f"  Note: Could not set dates (using defaults): {e}")
            
            # Click "Conferma" (Confirm) button
            print("\nClicking 'Conferma' button to download...")
            
            confirm_button = None
            confirm_selectors = [
                ("button:has-text('Conferma')", "Button with 'Conferma' text"),
                ("input[value='Conferma']", "Input with 'Conferma' value"),
                ("button.confirm, .btn-confirm, #btn-confirm", "Button with confirm class/id"),
            ]
            
            for selector, description in confirm_selectors:
                try:
                    confirm_button = page.locator(selector).first
                    if confirm_button.is_visible():
                        print(f"✓ Found 'Conferma' button: {description}")
                        confirm_button.click()
                        print("✓ Clicked 'Conferma' - download should start...")
                        break
                except:
                    continue
            
            if confirm_button is None or not confirm_button.is_visible():
                print("❌ Could not find 'Conferma' button automatically")
                print(f"   Current URL: {page.url}")
                
                # Try to find what buttons are available
                try:
                    all_buttons = page.locator("button, input[type='button'], input[type='submit']").all()
                    print(f"   Found {len(all_buttons)} buttons on page:")
                    for i, btn in enumerate(all_buttons[:10]):  # Show first 10
                        try:
                            text = btn.text_content() or btn.get_attribute("value") or ""
                            if text.strip():
                                print(f"     - Button {i+1}: '{text.strip()}'")
                        except:
                            pass
                except Exception as e:
                    print(f"   Could not list buttons: {e}")
                
                if interactive:
                    print("\nBrowser is open - please manually click 'Conferma' in the modal.")
                    print("Press Enter when download starts...")
                    input()
                else:
                    screenshot_path = download_dir / "classeviva_modal_screenshot.png"
                    try:
                        page.screenshot(path=str(screenshot_path), full_page=True)
                        print(f"📸 Screenshot saved to: {screenshot_path}")
                    except Exception as e:
                        print(f"   Could not save screenshot: {e}")
                    print("   Try running with --interactive to see what's on the page")
                    return False
            
            # Wait for download to start
            time.sleep(2)
        
        # Get the download
        print("Waiting for download to complete...")
        try:
            download = download_info.value
            print(f"✓ Download started: {download.suggested_filename}")
        except Exception as e:
            print(f"❌ Failed to get download object: {e}")
            return False
        
        # Save the file
        timestamp = datetime.now().strftime("%Y%m%d")
        filename = download_dir / f"export_{timestamp}.xls"
        
        print(f"Saving download to: {filename}")
        if filename.exists():
            print(f"   (Removing existing file: {filename})")
            filename.unlink()
        
        try:
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
        except Exception as e:
            print(f"❌ Error saving download: {e}")
            import traceback
            traceback.print_exc()
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
