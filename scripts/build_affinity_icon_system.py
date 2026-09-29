#!/usr/bin/env python3
"""
build_affinity_icon_system.py
Generates 100% original bespoke vector icons for the Proteus Design System in Affinity Designer,
replacing generic system emojis with scalable, dynamic-resolution vector symbols.
Also embeds the official King Proteus emblem and authoritative color palette across the master artboard and icon sheets.
"""

import os
import sys
import xml.etree.ElementTree as ET

BASE_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
AFFINITY_DIR = os.path.join(BASE_DIR, "affinity-assets")
ICONS_DIR = os.path.join(AFFINITY_DIR, "icons")
LOGOS_DIR = os.path.join(AFFINITY_DIR, "logos")
WEB_ICONS_DIR = os.path.join(BASE_DIR, "project", "proteus-web", "assets", "icons")
PROJECT_ICONS_DIR = os.path.join(BASE_DIR, "project", "assets", "icons")

for d in [ICONS_DIR, LOGOS_DIR, WEB_ICONS_DIR, PROJECT_ICONS_DIR]:
    os.makedirs(d, exist_ok=True)

# 13 Bespoke Vector Icons replacing generic emojis
# Built on 24x24 viewBox with 1.75 stroke-width, stroke="currentColor", fill="none"
BESPOKE_ICONS = {
    "icon_dashboard": {
        "name": "icon_dashboard.svg",
        "label": "Analytics / Dashboard (replacing 📊)",
        "svg": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
  <path d="M3 3v18h18"/>
  <rect x="7" y="10" width="3" height="8" rx="1"/>
  <rect x="13" y="6" width="3" height="12" rx="1"/>
  <rect x="19" y="13" width="3" height="5" rx="1"/>
</svg>"""
    },
    "icon_marketplace": {
        "name": "icon_marketplace.svg",
        "label": "Marketplace / Shop (replacing 🛒)",
        "svg": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
  <circle cx="9" cy="20" r="1.5"/>
  <circle cx="18" cy="20" r="1.5"/>
  <path d="M2.5 3h3l2.6 11.2a1.5 1.5 0 0 0 1.5 1.2h9.4a1.5 1.5 0 0 0 1.5-1.1L22 7H6.2"/>
</svg>"""
    },
    "icon_brief": {
        "name": "icon_brief.svg",
        "label": "Bespoke Brief / Audit (replacing 📋)",
        "svg": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
  <rect x="4" y="5" width="16" height="16" rx="2"/>
  <path d="M9 3h6a1 1 0 0 1 1 1v2a1 1 0 0 1-1 1H9a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1Z"/>
  <line x1="8" y1="11" x2="16" y2="11"/>
  <line x1="8" y1="15" x2="13" y2="15"/>
</svg>"""
    },
    "icon_contract": {
        "name": "icon_contract.svg",
        "label": "Digital SLA Contract (replacing 📜)",
        "svg": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
  <path d="M19 19a2 2 0 0 1-2 2H6a3 3 0 0 1-3-3V6a3 3 0 0 1 3-3h10a2 2 0 0 1 2 2v14Z"/>
  <path d="M18 17h2a2 2 0 0 1 2 2v0a2 2 0 0 1-2 2h-3"/>
  <line x1="7" y1="8" x2="14" y2="8"/>
  <line x1="7" y1="12" x2="14" y2="12"/>
  <line x1="7" y1="16" x2="11" y2="16"/>
</svg>"""
    },
    "icon_cert": {
        "name": "icon_cert.svg",
        "label": "Certification Academy (replacing 🎓)",
        "svg": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
  <path d="M22 10 12 5 2 10l10 5 10-5Z"/>
  <path d="M6 12v5c0 2.5 3 4 6 4s6-1.5 6-4v-5"/>
  <line x1="22" y1="10" x2="22" y2="16"/>
</svg>"""
    },
    "icon_domains": {
        "name": "icon_domains.svg",
        "label": "Domains & Hosting (replacing 🌐)",
        "svg": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
  <circle cx="12" cy="12" r="9"/>
  <line x1="3" y1="12" x2="21" y2="12"/>
  <path d="M12 3a14.5 14.5 0 0 0 0 18"/>
  <path d="M12 3a14.5 14.5 0 0 1 0 18"/>
</svg>"""
    },
    "icon_freelance": {
        "name": "icon_freelance.svg",
        "label": "Freelancing & Specialists (replacing 💼)",
        "svg": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
  <rect x="3" y="7" width="18" height="14" rx="2"/>
  <path d="M8 7V5a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
  <line x1="3" y1="12" x2="21" y2="12"/>
  <circle cx="12" cy="12" r="1.5"/>
</svg>"""
    },
    "icon_users": {
        "name": "icon_users.svg",
        "label": "Multi-Tenant & Account (replacing 👥)",
        "svg": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
  <circle cx="9" cy="7" r="4"/>
  <path d="M2 20c0-3.3 3.1-6 7-6s7 2.7 7 6"/>
  <path d="M16 3.5a3.5 3.5 0 0 1 0 7"/>
  <path d="M19 14.5c2 .7 3 2.3 3 4.5"/>
</svg>"""
    },
    "icon_package": {
        "name": "icon_package.svg",
        "label": "Proteus Package / Export (replacing 📦)",
        "svg": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
  <path d="M12 3 3.5 7.8v8.4L12 21l8.5-4.8V7.8L12 3Z"/>
  <line x1="12" y1="3" x2="12" y2="21"/>
  <line x1="3.5" y1="7.8" x2="12" y2="12.5"/>
  <line x1="20.5" y1="7.8" x2="12" y2="12.5"/>
</svg>"""
    },
    "icon_hardware": {
        "name": "icon_hardware.svg",
        "label": "Hardware Device / POS (replacing 🖨)",
        "svg": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
  <rect x="4" y="9" width="16" height="11" rx="2"/>
  <path d="M7 9V4a1 1 0 0 1 1-1h8a1 1 0 0 1 1 1v5"/>
  <line x1="8" y1="14" x2="16" y2="14"/>
  <circle cx="16" cy="17" r="1" fill="currentColor"/>
</svg>"""
    },
    "icon_search": {
        "name": "icon_search.svg",
        "label": "Search / Command Palette (replacing 🔍)",
        "svg": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
  <circle cx="11" cy="11" r="7"/>
  <line x1="16.5" y1="16.5" x2="21" y2="21"/>
</svg>"""
    },
    "icon_settings": {
        "name": "icon_settings.svg",
        "label": "Configuration / Settings (replacing ⚙)",
        "svg": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
  <circle cx="12" cy="12" r="3.5"/>
  <path d="M12 2v2.5M12 19.5V22M2 12h2.5M19.5 12H22M4.9 4.9l1.8 1.8M17.3 17.3l1.8 1.8M4.9 19.1l1.8-1.8M17.3 6.7l1.8-1.8"/>
</svg>"""
    },
    "icon_pulse": {
        "name": "icon_pulse.svg",
        "label": "Sync / Pulse / Telemetry (replacing ⚡)",
        "svg": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
  <polygon points="13 2 4 14 11 14 9 22 20 10 13 10 13 2"/>
</svg>"""
    }
}

# Write individual SVG files to all asset locations
for key, data in BESPOKE_ICONS.items():
    for target_dir in [ICONS_DIR, WEB_ICONS_DIR, PROJECT_ICONS_DIR]:
        path = os.path.join(target_dir, data["name"])
        with open(path, "w", encoding="utf-8") as f:
            f.write(data["svg"])

print(f"Generated {len(BESPOKE_ICONS)} bespoke SVG icons across asset directories.")

# Generate Master Affinity Designer Artboard SVG
MASTER_SVG_PATH = os.path.join(AFFINITY_DIR, "Proteus_Design_System_Master.svg")

# Read King Proteus emblem data
EMBLEM_SVG_PATH = os.path.join(LOGOS_DIR, "proteus_king_emblem.svg")
with open(EMBLEM_SVG_PATH, "r", encoding="utf-8") as f:
    emblem_svg_full = f.read()

import re
href_match = re.search(r'href="([^"]+)"', emblem_svg_full)
data_href = href_match.group(1) if href_match else ""

master_svg = f"""<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 1920 1820" width="1920" height="1820">
  <defs>
    <linearGradient id="mSlate" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#DDE6ED"/>
      <stop offset="100%" stop-color="#9DB2BF"/>
    </linearGradient>
    <linearGradient id="mWave" x1="0%" y1="0%" x2="100%" y2="0%">
      <stop offset="0%" stop-color="#265863"/>
      <stop offset="50%" stop-color="#526D82"/>
      <stop offset="100%" stop-color="#DDE6ED"/>
    </linearGradient>
    <linearGradient id="goldOchre" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#EED9B3"/>
      <stop offset="100%" stop-color="#C5A880"/>
    </linearGradient>
    <filter id="cardShadow" x="-5%" y="-5%" width="110%" height="110%">
      <feDropShadow dx="0" dy="4" stdDeviation="6" flood-color="#000000" flood-opacity="0.3"/>
    </filter>
  </defs>

  <!-- Background -->
  <rect width="1920" height="1820" fill="#11161F"/>

  <!-- Master Header -->
  <g id="Header" transform="translate(60, 60)">
    <text x="0" y="0" font-family="Inter, -apple-system, sans-serif" font-size="28" font-weight="800" fill="#DDE6ED" letter-spacing="2">PROTEUS DESIGN SYSTEM // AFFINITY MASTER ASSET SUITE</text>
    <text x="0" y="28" font-family="Inter, -apple-system, sans-serif" font-size="13" font-weight="500" fill="#9DB2BF">100% Bespoke Original Vector Architecture — King Proteus Sovereign Emblem, Authoritative Color Palette &amp; Scalable Glyph System</text>
  </g>

  <!-- ========================================== -->
  <!-- 00. KING PROTEUS SOVEREIGN EMBLEM & RESOLUTION SHEET -->
  <!-- ========================================== -->
  <g id="00. KING PROTEUS SOVEREIGN EMBLEM">
    <rect x="60" y="110" width="1800" height="280" rx="14" fill="#18202C" stroke="#253549" stroke-width="1.5" filter="url(#cardShadow)"/>
    <text x="84" y="145" font-family="Inter, sans-serif" font-size="15" font-weight="800" fill="#DDE6ED" letter-spacing="1">00. KING PROTEUS SOVEREIGN EMBLEM // MULTI-SCALE RESOLUTION MATRIX</text>
    <text x="84" y="165" font-family="Inter, sans-serif" font-size="11" font-weight="500" fill="#9DB2BF">Primary Brand Mark: Regal Crown with Seashell Center &amp; Marine Beard (#265863 Petrol Teal + #F5EED4 Pale Cream)</text>

    <!-- 256x256 Master -->
    <g transform="translate(90, 185)">
      <rect width="180" height="180" rx="16" fill="#11161F" stroke="#265863" stroke-width="2"/>
      <image xlink:href="logos/proteus_emblem_256.png" href="{data_href}" x="10" y="10" width="160" height="160"/>
      <text x="90" y="198" font-family="Inter, sans-serif" font-size="11" font-weight="700" fill="#DDE6ED" text-anchor="middle">256×256 Master</text>
    </g>

    <!-- 128x128 Window Icon -->
    <g transform="translate(300, 205)">
      <rect width="130" height="130" rx="14" fill="#11161F" stroke="#3A4D63" stroke-width="1.5"/>
      <image xlink:href="logos/proteus_emblem_128.png" href="{data_href}" x="15" y="15" width="100" height="100"/>
      <text x="65" y="155" font-family="Inter, sans-serif" font-size="11" font-weight="700" fill="#DDE6ED" text-anchor="middle">128×128 Window Icon</text>
    </g>

    <!-- 64x64 Rail -->
    <g transform="translate(460, 230)">
      <rect width="84" height="84" rx="10" fill="#11161F" stroke="#3A4D63" stroke-width="1.5"/>
      <image xlink:href="logos/proteus_emblem_64.png" href="{data_href}" x="10" y="10" width="64" height="64"/>
      <text x="42" y="106" font-family="Inter, sans-serif" font-size="10" font-weight="600" fill="#9DB2BF" text-anchor="middle">64×64 Rail</text>
    </g>

    <!-- 32x32 Header -->
    <g transform="translate(570, 245)">
      <rect width="56" height="56" rx="8" fill="#11161F" stroke="#3A4D63" stroke-width="1.5"/>
      <image xlink:href="logos/proteus_emblem_32.png" href="{data_href}" x="12" y="12" width="32" height="32"/>
      <text x="28" y="78" font-family="Inter, sans-serif" font-size="10" font-weight="600" fill="#9DB2BF" text-anchor="middle">32×32 TopBar</text>
    </g>

    <!-- 24x24 Toolbar -->
    <g transform="translate(650, 255)">
      <rect width="44" height="44" rx="6" fill="#11161F" stroke="#3A4D63" stroke-width="1.5"/>
      <image xlink:href="logos/proteus_emblem_24.png" href="{data_href}" x="10" y="10" width="24" height="24"/>
      <text x="22" y="66" font-family="Inter, sans-serif" font-size="9" font-weight="600" fill="#9DB2BF" text-anchor="middle">24×24 Tool</text>
    </g>

    <!-- 16x16 Favicon -->
    <g transform="translate(718, 263)">
      <rect width="34" height="34" rx="4" fill="#11161F" stroke="#3A4D63" stroke-width="1.5"/>
      <image xlink:href="logos/proteus_emblem_16.png" href="{data_href}" x="9" y="9" width="16" height="16"/>
      <text x="17" y="54" font-family="Inter, sans-serif" font-size="9" font-weight="600" fill="#9DB2BF" text-anchor="middle">16×16 Ico</text>
    </g>

    <!-- Dynamic DPI Scaling Table -->
    <g transform="translate(800, 150)">
      <rect width="500" height="215" rx="10" fill="#131B26" stroke="#253549" stroke-width="1"/>
      <text x="20" y="28" font-family="Inter, sans-serif" font-size="12" font-weight="800" fill="#DDE6ED">DYNAMIC DPI RESOLUTION ADAPTATION GUIDELINES</text>
      
      <text x="20" y="60" font-family="Inter, sans-serif" font-size="11" font-weight="600" fill="#9DB2BF">100% (96 DPI Standard):</text>
      <text x="220" y="60" font-family="JetBrains Mono, monospace" font-size="11" fill="#38BDF8">24px Grid / 1.75px Stroke / 1.0x Scale</text>

      <text x="20" y="95" font-family="Inter, sans-serif" font-size="11" font-weight="600" fill="#9DB2BF">125% (120 DPI Windows):</text>
      <text x="220" y="95" font-family="JetBrains Mono, monospace" font-size="11" fill="#38BDF8">30px Grid / 2.18px Stroke / 1.25x Scale</text>

      <text x="20" y="130" font-family="Inter, sans-serif" font-size="11" font-weight="600" fill="#9DB2BF">150% (144 DPI QHD/Laptop):</text>
      <text x="220" y="130" font-family="JetBrains Mono, monospace" font-size="11" fill="#38BDF8">36px Grid / 2.62px Stroke / 1.5x Scale</text>

      <text x="20" y="165" font-family="Inter, sans-serif" font-size="11" font-weight="600" fill="#9DB2BF">200% (192 DPI 4K / Retina):</text>
      <text x="220" y="165" font-family="JetBrains Mono, monospace" font-size="11" fill="#38BDF8">48px Grid / 3.50px Stroke / 2.0x Scale</text>

      <text x="20" y="195" font-family="Inter, sans-serif" font-size="10" fill="#526D82">Vector stroke-width scales linearly with canvas zoom and DPI viewport ratio.</text>
    </g>

    <!-- Brand Palette Swatches -->
    <g transform="translate(1330, 150)">
      <rect width="500" height="215" rx="10" fill="#131B26" stroke="#253549" stroke-width="1"/>
      <text x="20" y="28" font-family="Inter, sans-serif" font-size="12" font-weight="800" fill="#DDE6ED">OFFICIAL BRAND COLOR SPECIFICATIONS</text>

      <rect x="20" y="50" width="34" height="34" rx="6" fill="#265863"/>
      <text x="65" y="65" font-family="Inter, sans-serif" font-size="11" font-weight="700" fill="#DDE6ED">#265863 — Deep Petrol King Teal</text>
      <text x="65" y="80" font-family="Inter, sans-serif" font-size="10" fill="#9DB2BF">Primary Brand Anchor, Regal Beard, Outlines</text>

      <rect x="20" y="100" width="34" height="34" rx="6" fill="#F5EED4"/>
      <text x="65" y="115" font-family="Inter, sans-serif" font-size="11" font-weight="700" fill="#DDE6ED">#F5EED4 — Pale Cream Shell</text>
      <text x="65" y="130" font-family="Inter, sans-serif" font-size="10" fill="#9DB2BF">Face Highlights, Crown Band, Contrast Surfaces</text>

      <rect x="20" y="150" width="34" height="34" rx="6" fill="#C5A880"/>
      <text x="65" y="165" font-family="Inter, sans-serif" font-size="11" font-weight="700" fill="#DDE6ED">#C5A880 — Warm Ochre Gold</text>
      <text x="65" y="180" font-family="Inter, sans-serif" font-size="10" fill="#9DB2BF">Proteus Client Monogram Accent &amp; VIP Badges</text>
    </g>
  </g>

  <!-- ========================================== -->
  <!-- 01. AUTHORITATIVE MASTER COLOR PALETTE & BRAND HARMONY -->
  <!-- ========================================== -->
  <g id="01. AUTHORITATIVE MASTER COLOR PALETTE &amp; BRAND HARMONY">
    <rect x="60" y="410" width="1800" height="330" rx="14" fill="#18202C" stroke="#253549" stroke-width="1.5" filter="url(#cardShadow)"/>
    <text x="84" y="442" font-family="Inter, sans-serif" font-size="15" font-weight="800" fill="#DDE6ED" letter-spacing="1">01. MASTER COLOR PALETTE &amp; BRAND HARMONY // DESIGN TOKENS &amp; SWATCHES</text>
    <text x="84" y="462" font-family="Inter, sans-serif" font-size="11" font-weight="500" fill="#9DB2BF">Complete Color Architecture: King Proteus Sovereign Brand + Nordic Slate Foundations + Obsidian Dark + Semantic Accents</text>

    <!-- Column 1: King Proteus Brand Identity -->
    <g transform="translate(85, 480)">
      <rect width="405" height="240" rx="10" fill="#131B26" stroke="#253549" stroke-width="1"/>
      <text x="20" y="24" font-family="Inter, sans-serif" font-size="11" font-weight="800" fill="#DDE6ED" letter-spacing="1">A. KING PROTEUS BRAND IDENTITY</text>

      <!-- Swatch 1: Petrol King Teal -->
      <g transform="translate(20, 36)">
        <rect width="44" height="34" rx="6" fill="#265863" stroke="#3A4D63" stroke-width="1"/>
        <text x="56" y="16" font-family="Inter, sans-serif" font-size="11" font-weight="700" fill="#DDE6ED">#265863 — Petrol King Teal</text>
        <text x="56" y="30" font-family="Inter, sans-serif" font-size="9.5" fill="#9DB2BF">Primary Brand Anchor, Beard &amp; Outlines</text>
      </g>

      <!-- Swatch 2: Pale Cream Shell -->
      <g transform="translate(20, 84)">
        <rect width="44" height="34" rx="6" fill="#F5EED4" stroke="#C5A880" stroke-width="1"/>
        <text x="56" y="16" font-family="Inter, sans-serif" font-size="11" font-weight="700" fill="#DDE6ED">#F5EED4 — Pale Cream Shell</text>
        <text x="56" y="30" font-family="Inter, sans-serif" font-size="9.5" fill="#9DB2BF">Face Highlights, Crown Band &amp; Contrast</text>
      </g>

      <!-- Swatch 3: Warm Ochre Gold -->
      <g transform="translate(20, 132)">
        <rect width="44" height="34" rx="6" fill="#C5A880" stroke="#EED9B3" stroke-width="1"/>
        <text x="56" y="16" font-family="Inter, sans-serif" font-size="11" font-weight="700" fill="#DDE6ED">#C5A880 — Warm Ochre Gold</text>
        <text x="56" y="30" font-family="Inter, sans-serif" font-size="9.5" fill="#9DB2BF">Client Monogram Accent &amp; VIP Badges</text>
      </g>

      <!-- Swatch 4: Sky Cyan Pulse -->
      <g transform="translate(20, 180)">
        <rect width="44" height="34" rx="6" fill="#38BDF8" stroke="#0284C7" stroke-width="1"/>
        <text x="56" y="16" font-family="Inter, sans-serif" font-size="11" font-weight="700" fill="#DDE6ED">#38BDF8 — Sky Cyan Glow</text>
        <text x="56" y="30" font-family="Inter, sans-serif" font-size="9.5" fill="#9DB2BF">Active Focus Rings, Glow &amp; Pulse Node</text>
      </g>
    </g>

    <!-- Column 2: Original Obsidian Luxury Minimalist Palette -->
    <g transform="translate(510, 480)">
      <rect width="415" height="240" rx="10" fill="#131B26" stroke="#253549" stroke-width="1"/>
      <text x="20" y="24" font-family="Inter, sans-serif" font-size="11" font-weight="800" fill="#DDE6ED" letter-spacing="1">B. ORIGINAL MINIMALIST PALETTE</text>

      <!-- Swatch 1: Obsidian Base -->
      <g transform="translate(20, 36)">
        <rect width="44" height="34" rx="6" fill="#0C0E14" stroke="#262C3D" stroke-width="1"/>
        <text x="56" y="16" font-family="Inter, sans-serif" font-size="11" font-weight="700" fill="#DDE6ED">#0C0E14 / #0E0F12 — Obsidian Base</text>
        <text x="56" y="30" font-family="Inter, sans-serif" font-size="9.5" fill="#9DB2BF">Original Ultra-Dark Canvas Background</text>
      </g>

      <!-- Swatch 2: Panel Surface -->
      <g transform="translate(20, 84)">
        <rect width="44" height="34" rx="6" fill="#151821" stroke="#262C3D" stroke-width="1"/>
        <text x="56" y="16" font-family="Inter, sans-serif" font-size="11" font-weight="700" fill="#DDE6ED">#151821 / #14151A — Panel Surface</text>
        <text x="56" y="30" font-family="Inter, sans-serif" font-size="9.5" fill="#9DB2BF">Sleek Panel Surfaces &amp; Left Navigation</text>
      </g>

      <!-- Swatch 3: Elevated Card -->
      <g transform="translate(20, 132)">
        <rect width="44" height="34" rx="6" fill="#1B202E" stroke="#262C3D" stroke-width="1"/>
        <text x="56" y="16" font-family="Inter, sans-serif" font-size="11" font-weight="700" fill="#DDE6ED">#1B202E / #1E2436 — Elevated Card</text>
        <text x="56" y="30" font-family="Inter, sans-serif" font-size="9.5" fill="#9DB2BF">Floating Windows, Dialogs &amp; Dropdowns</text>
      </g>

      <!-- Swatch 4: Primary Brand Blue & Border -->
      <g transform="translate(20, 180)">
        <rect width="21" height="34" rx="4" fill="#3B82F6" stroke="#2563EB" stroke-width="1"/>
        <rect x="23" width="21" height="34" rx="4" fill="#262C3D" stroke="#3B4661" stroke-width="1"/>
        <text x="56" y="16" font-family="Inter, sans-serif" font-size="11" font-weight="700" fill="#DDE6ED">#3B82F6 Blue + #262C3D Border</text>
        <text x="56" y="30" font-family="Inter, sans-serif" font-size="9.5" fill="#9DB2BF">Original Primary CTA &amp; Crisp 1px Hairline</text>
      </g>
    </g>

    <!-- Column 3: Nordic Slate Architecture Foundations -->
    <g transform="translate(945, 480)">
      <rect width="425" height="240" rx="10" fill="#131B26" stroke="#253549" stroke-width="1"/>
      <text x="20" y="24" font-family="Inter, sans-serif" font-size="11" font-weight="800" fill="#DDE6ED" letter-spacing="1">C. NORDIC SLATE ARCHITECTURE</text>

      <!-- Swatch 1: Canvas Dark & Secondary -->
      <g transform="translate(20, 36)">
        <rect width="21" height="34" rx="4" fill="#1B2430" stroke="#3A4D63" stroke-width="1"/>
        <rect x="23" width="21" height="34" rx="4" fill="#1E2A3A" stroke="#526D82" stroke-width="1"/>
        <text x="56" y="16" font-family="Inter, sans-serif" font-size="11" font-weight="700" fill="#DDE6ED">#1B2430 + #1E2A3A — Canvas &amp; Panels</text>
        <text x="56" y="30" font-family="Inter, sans-serif" font-size="9.5" fill="#9DB2BF">Slate Canvas &amp; Recessed Toolbars</text>
      </g>

      <!-- Swatch 2: Deep Slate Navy -->
      <g transform="translate(20, 84)">
        <rect width="44" height="34" rx="6" fill="#27374D" stroke="#526D82" stroke-width="1"/>
        <text x="56" y="16" font-family="Inter, sans-serif" font-size="11" font-weight="700" fill="#DDE6ED">#27374D — Deep Slate Navy</text>
        <text x="56" y="30" font-family="Inter, sans-serif" font-size="9.5" fill="#9DB2BF">Elevated Cards &amp; Container Surfaces</text>
      </g>

      <!-- Swatch 3: Steel Slate & Border Subtle -->
      <g transform="translate(20, 132)">
        <rect width="44" height="34" rx="6" fill="#526D82" stroke="#9DB2BF" stroke-width="1"/>
        <text x="56" y="16" font-family="Inter, sans-serif" font-size="11" font-weight="700" fill="#DDE6ED">#526D82 — Steel Slate (1px Border)</text>
        <text x="56" y="30" font-family="Inter, sans-serif" font-size="9.5" fill="#9DB2BF">1px Hairline Grid &amp; Division Tokens</text>
      </g>

      <!-- Swatch 4: Icy Mist & Frost Gray -->
      <g transform="translate(20, 180)">
        <rect width="21" height="34" rx="4" fill="#DDE6ED" stroke="#526D82" stroke-width="1"/>
        <rect x="23" width="21" height="34" rx="4" fill="#9DB2BF" stroke="#526D82" stroke-width="1"/>
        <text x="56" y="16" font-family="Inter, sans-serif" font-size="11" font-weight="700" fill="#DDE6ED">#DDE6ED + #9DB2BF — Text Tokens</text>
        <text x="56" y="30" font-family="Inter, sans-serif" font-size="9.5" fill="#9DB2BF">Primary Headings &amp; Secondary Muted</text>
      </g>
    </g>

    <!-- Column 4: Semantic Feedback & Accents -->
    <g transform="translate(1390, 480)">
      <rect width="450" height="240" rx="10" fill="#131B26" stroke="#253549" stroke-width="1"/>
      <text x="20" y="24" font-family="Inter, sans-serif" font-size="11" font-weight="800" fill="#DDE6ED" letter-spacing="1">D. SEMANTIC ACCENTS &amp; FEEDBACK</text>

      <!-- Swatch 1: Indigo -->
      <g transform="translate(20, 36)">
        <rect width="36" height="28" rx="5" fill="#6366F1"/>
        <text x="48" y="14" font-family="Inter, sans-serif" font-size="10.5" font-weight="700" fill="#DDE6ED">#6366F1 — Linear Indigo</text>
        <text x="48" y="26" font-family="Inter, sans-serif" font-size="9" fill="#9DB2BF">Creative &amp; Studio Accent</text>
      </g>

      <!-- Swatch 2: Sapphire -->
      <g transform="translate(20, 74)">
        <rect width="36" height="28" rx="5" fill="#2563EB"/>
        <text x="48" y="14" font-family="Inter, sans-serif" font-size="10.5" font-weight="700" fill="#DDE6ED">#2563EB — Sapphire Blue</text>
        <text x="48" y="26" font-family="Inter, sans-serif" font-size="9" fill="#9DB2BF">Enterprise Links &amp; System CTAs</text>
      </g>

      <!-- Swatch 3: Emerald -->
      <g transform="translate(20, 112)">
        <rect width="36" height="28" rx="5" fill="#34D399"/>
        <text x="48" y="14" font-family="Inter, sans-serif" font-size="10.5" font-weight="700" fill="#DDE6ED">#34D399 / #10B981 — Emerald Green</text>
        <text x="48" y="26" font-family="Inter, sans-serif" font-size="9" fill="#9DB2BF">Success, Mounted Status &amp; Spooler Beacon</text>
      </g>

      <!-- Swatch 4: Amber -->
      <g transform="translate(20, 150)">
        <rect width="36" height="28" rx="5" fill="#F59E0B"/>
        <text x="48" y="14" font-family="Inter, sans-serif" font-size="10.5" font-weight="700" fill="#DDE6ED">#F59E0B — Amber Gold</text>
        <text x="48" y="26" font-family="Inter, sans-serif" font-size="9" fill="#9DB2BF">Warning, In-Progress &amp; Review Notice</text>
      </g>

      <!-- Swatch 5: Red -->
      <g transform="translate(20, 188)">
        <rect width="36" height="28" rx="5" fill="#EF4444"/>
        <text x="48" y="14" font-family="Inter, sans-serif" font-size="10.5" font-weight="700" fill="#DDE6ED">#EF4444 — Coral Red</text>
        <text x="48" y="26" font-family="Inter, sans-serif" font-size="9" fill="#9DB2BF">Critical Alert, Danger &amp; Barcode Laser</text>
      </g>
    </g>
  </g>

  <!-- ========================================== -->
  <!-- 02. BRAND IDENTITY & MONOGRAMS -->
  <!-- ========================================== -->
  <g id="02. BRAND IDENTITY &amp; MONOGRAMS">
    <rect x="60" y="760" width="560" height="240" rx="12" fill="#18202C" stroke="#253549" stroke-width="1.5"/>
    <text x="84" y="790" font-family="Inter, sans-serif" font-size="14" font-weight="800" fill="#DDE6ED">02. BRAND IDENTITY &amp; MONOGRAMS</text>

    <!-- PDS Monogram -->
    <g transform="translate(85, 815) scale(0.9)">
      <rect width="128" height="128" rx="24" fill="#202D3E" stroke="#526D82" stroke-width="2"/>
      <line x1="36" y1="20" x2="36" y2="76" stroke="url(#mSlate)" stroke-width="12" stroke-linecap="round"/>
      <path d="M 36 20 L 70 20 C 88 20 88 50 70 50 L 36 50" fill="none" stroke="url(#mSlate)" stroke-width="12" stroke-linecap="round" stroke-linejoin="round"/>
      <path d="M 22 92 C 44 80 74 104 106 88" fill="none" stroke="url(#mWave)" stroke-width="9" stroke-linecap="round"/>
      <path d="M 22 108 C 44 96 74 120 106 104" fill="none" stroke="url(#mWave)" stroke-width="9" stroke-linecap="round" opacity="0.85"/>
    </g>
    <text x="220" y="870" font-family="Inter, sans-serif" font-size="28" font-weight="800" fill="#DDE6ED" letter-spacing="3">PROTEUS</text>
    <text x="222" y="895" font-family="Inter, sans-serif" font-size="11" font-weight="600" fill="#9DB2BF" letter-spacing="2">THE VISUAL OS FOR BUSINESS</text>
    <text x="222" y="920" font-family="Inter, sans-serif" font-size="10" fill="#526D82">PDS Stylized Monogram + Dual Wave</text>
  </g>

  <!-- ========================================== -->
  <!-- 03. 5-ROLE ECOSYSTEM EMBLEMS -->
  <!-- ========================================== -->
  <g id="03. 5-ROLE ECOSYSTEM BADGES">
    <rect x="650" y="760" width="1210" height="240" rx="12" fill="#18202C" stroke="#253549" stroke-width="1.5"/>
    <text x="674" y="790" font-family="Inter, sans-serif" font-size="14" font-weight="800" fill="#DDE6ED">03. 5-ROLE ECOSYSTEM BADGES (BESPOKE VECTOR NODES)</text>

    <!-- PCD -->
    <g transform="translate(680, 815)">
      <rect width="90" height="90" rx="16" fill="#1E2A3A" stroke="#526D82" stroke-width="2"/>
      <path d="M 32 60 L 32 46 L 54 24 C 57 21 62 21 65 24 C 68 27 68 32 65 35 L 43 57 L 32 60 Z" fill="none" stroke="#DDE6ED" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"/>
      <circle cx="48" cy="40" r="2.5" fill="#DDE6ED"/>
      <text x="45" y="120" font-family="Inter, sans-serif" font-size="12" font-weight="700" fill="#DDE6ED" text-anchor="middle">PCD</text>
      <text x="45" y="136" font-family="Inter, sans-serif" font-size="10" fill="#9DB2BF" text-anchor="middle">UI Designer</text>
    </g>

    <!-- PCDA -->
    <g transform="translate(830, 815)">
      <rect width="90" height="90" rx="16" fill="#1E2A3A" stroke="#526D82" stroke-width="2"/>
      <rect x="20" y="20" width="20" height="16" rx="3" fill="#27374D" stroke="#DDE6ED" stroke-width="2"/>
      <rect x="50" y="20" width="20" height="16" rx="3" fill="#27374D" stroke="#DDE6ED" stroke-width="2"/>
      <rect x="35" y="52" width="20" height="16" rx="3" fill="#27374D" stroke="#9DB2BF" stroke-width="2"/>
      <path d="M 30 36 L 30 44 L 45 44 L 45 52" fill="none" stroke="#526D82" stroke-width="2"/>
      <path d="M 60 36 L 60 44 L 45 44" fill="none" stroke="#526D82" stroke-width="2"/>
      <text x="45" y="120" font-family="Inter, sans-serif" font-size="12" font-weight="700" fill="#DDE6ED" text-anchor="middle">PCDA</text>
      <text x="45" y="136" font-family="Inter, sans-serif" font-size="10" fill="#9DB2BF" text-anchor="middle">Data Analyst</text>
    </g>

    <!-- PCSS -->
    <g transform="translate(980, 815)">
      <rect width="90" height="90" rx="16" fill="#1E2A3A" stroke="#526D82" stroke-width="2"/>
      <circle cx="45" cy="46" r="8" fill="#DDE6ED"/>
      <path d="M 32 34 C 38 27 52 27 58 34" fill="none" stroke="#9DB2BF" stroke-width="3" stroke-linecap="round"/>
      <path d="M 24 26 C 36 15 54 15 66 26" fill="none" stroke="#526D82" stroke-width="3" stroke-linecap="round"/>
      <line x1="45" y1="54" x2="45" y2="68" stroke="#526D82" stroke-width="3"/>
      <text x="45" y="120" font-family="Inter, sans-serif" font-size="12" font-weight="700" fill="#DDE6ED" text-anchor="middle">PCSS</text>
      <text x="45" y="136" font-family="Inter, sans-serif" font-size="10" fill="#9DB2BF" text-anchor="middle">Systems &amp; IT</text>
    </g>

    <!-- PCDS -->
    <g transform="translate(1130, 815)">
      <rect width="90" height="90" rx="16" fill="#1E2A3A" stroke="#526D82" stroke-width="2"/>
      <rect x="26" y="32" width="38" height="32" rx="5" fill="#27374D" stroke="#DDE6ED" stroke-width="2.5"/>
      <path d="M 34 32 L 34 22 L 56 22 L 56 32" fill="none" stroke="#9DB2BF" stroke-width="2.5"/>
      <line x1="34" y1="44" x2="56" y2="44" stroke="#DDE6ED" stroke-width="2.5" stroke-linecap="round"/>
      <circle cx="56" cy="52" r="2" fill="#34D399"/>
      <text x="45" y="120" font-family="Inter, sans-serif" font-size="12" font-weight="700" fill="#DDE6ED" text-anchor="middle">PCDS</text>
      <text x="45" y="136" font-family="Inter, sans-serif" font-size="10" fill="#9DB2BF" text-anchor="middle">Hardware Field</text>
    </g>

    <!-- HQ -->
    <g transform="translate(1280, 815)">
      <rect width="90" height="90" rx="16" fill="#1E2A3A" stroke="#526D82" stroke-width="2"/>
      <path d="M 45 20 L 64 28 L 64 48 C 64 62 45 70 45 70 C 45 70 26 62 26 48 L 26 28 Z" fill="#27374D" stroke="#DDE6ED" stroke-width="3" stroke-linejoin="round"/>
      <line x1="40" y1="34" x2="40" y2="52" stroke="#DDE6ED" stroke-width="2.5" stroke-linecap="round"/>
      <path d="M 40 34 L 48 34 C 54 34 54 44 48 44 L 40 44" fill="none" stroke="#DDE6ED" stroke-width="2.5"/>
      <text x="45" y="120" font-family="Inter, sans-serif" font-size="12" font-weight="700" fill="#DDE6ED" text-anchor="middle">HQ</text>
      <text x="45" y="136" font-family="Inter, sans-serif" font-size="10" fill="#9DB2BF" text-anchor="middle">Store Director</text>
    </g>
  </g>

  <!-- ========================================== -->
  <!-- 04. WORKSPACE & TOOLBAR CONTROLS (24px / 48px) -->
  <!-- ========================================== -->
  <g id="04. UI STUDIO TOOLBAR CONTROLS">
    <rect x="60" y="1020" width="1800" height="235" rx="12" fill="#18202C" stroke="#253549" stroke-width="1.5"/>
    <text x="84" y="1050" font-family="Inter, sans-serif" font-size="14" font-weight="800" fill="#DDE6ED">04. UI STUDIO TOOLBAR CONTROLS (24px / 48px GRID)</text>

    <!-- Select -->
    <g transform="translate(100, 1085)">
      <circle cx="28" cy="28" r="28" fill="#1E2A3A" stroke="#526D82" stroke-width="1"/>
      <path d="M 19 14 L 30 38 L 33.5 28.5 L 43 25 Z" fill="#DDE6ED" stroke="#1B2430" stroke-width="1.8" stroke-linejoin="round"/>
      <text x="28" y="76" font-family="Inter, sans-serif" font-size="11" fill="#DDE6ED" text-anchor="middle">Select (V)</text>
    </g>
    <!-- Frame -->
    <g transform="translate(240, 1085)">
      <circle cx="28" cy="28" r="28" fill="#1E2A3A" stroke="#526D82" stroke-width="1"/>
      <line x1="20" y1="12" x2="20" y2="44" stroke="#9DB2BF" stroke-width="3"/>
      <line x1="36" y1="12" x2="36" y2="44" stroke="#9DB2BF" stroke-width="3"/>
      <line x1="12" y1="20" x2="44" y2="20" stroke="#9DB2BF" stroke-width="3"/>
      <line x1="12" y1="36" x2="44" y2="36" stroke="#9DB2BF" stroke-width="3"/>
      <text x="28" y="76" font-family="Inter, sans-serif" font-size="11" fill="#DDE6ED" text-anchor="middle">Frame (F)</text>
    </g>
    <!-- Text -->
    <g transform="translate(380, 1085)">
      <circle cx="28" cy="28" r="28" fill="#1E2A3A" stroke="#526D82" stroke-width="1"/>
      <path d="M 16 18 L 40 18 M 28 18 L 28 38 M 22 38 L 34 38" stroke="#DDE6ED" stroke-width="3.5" stroke-linecap="round"/>
      <text x="28" y="76" font-family="Inter, sans-serif" font-size="11" fill="#DDE6ED" text-anchor="middle">Text (T)</text>
    </g>
    <!-- Table -->
    <g transform="translate(520, 1085)">
      <circle cx="28" cy="28" r="28" fill="#1E2A3A" stroke="#526D82" stroke-width="1"/>
      <rect x="14" y="16" width="28" height="24" rx="3" fill="none" stroke="#DDE6ED" stroke-width="2.5"/>
      <line x1="14" y1="24" x2="42" y2="24" stroke="#9DB2BF" stroke-width="2"/>
      <line x1="23" y1="16" x2="23" y2="40" stroke="#9DB2BF" stroke-width="2"/>
      <line x1="33" y1="16" x2="33" y2="40" stroke="#9DB2BF" stroke-width="2"/>
      <text x="28" y="76" font-family="Inter, sans-serif" font-size="11" fill="#DDE6ED" text-anchor="middle">Table (G)</text>
    </g>
    <!-- Flow -->
    <g transform="translate(660, 1085)">
      <circle cx="28" cy="28" r="28" fill="#1E2A3A" stroke="#526D82" stroke-width="1"/>
      <circle cx="18" cy="20" r="4.5" fill="#27374D" stroke="#DDE6ED" stroke-width="2.5"/>
      <circle cx="38" cy="36" r="4.5" fill="#27374D" stroke="#DDE6ED" stroke-width="2.5"/>
      <path d="M 23 20 C 31 20 25 36 33 36" fill="none" stroke="#9DB2BF" stroke-width="3" stroke-linecap="round"/>
      <text x="28" y="76" font-family="Inter, sans-serif" font-size="11" fill="#DDE6ED" text-anchor="middle">Flows</text>
    </g>
    <!-- Zoom -->
    <g transform="translate(800, 1085)">
      <circle cx="28" cy="28" r="28" fill="#1E2A3A" stroke="#526D82" stroke-width="1"/>
      <circle cx="24" cy="24" r="9" fill="none" stroke="#DDE6ED" stroke-width="2.5"/>
      <line x1="31" y1="31" x2="40" y2="40" stroke="#DDE6ED" stroke-width="3.5" stroke-linecap="round"/>
      <text x="28" y="76" font-family="Inter, sans-serif" font-size="11" fill="#DDE6ED" text-anchor="middle">Zoom (Z)</text>
    </g>
    <!-- Ruler -->
    <g transform="translate(940, 1085)">
      <circle cx="28" cy="28" r="28" fill="#1E2A3A" stroke="#526D82" stroke-width="1"/>
      <rect x="12" y="20" width="32" height="16" rx="2" fill="none" stroke="#DDE6ED" stroke-width="2.5"/>
      <line x1="18" y1="20" x2="18" y2="26" stroke="#9DB2BF" stroke-width="2"/>
      <line x1="24" y1="20" x2="24" y2="29" stroke="#DDE6ED" stroke-width="2"/>
      <line x1="30" y1="20" x2="30" y2="26" stroke="#9DB2BF" stroke-width="2"/>
      <line x1="36" y1="20" x2="36" y2="29" stroke="#DDE6ED" stroke-width="2"/>
      <text x="28" y="76" font-family="Inter, sans-serif" font-size="11" fill="#DDE6ED" text-anchor="middle">Ruler (R)</text>
    </g>
  </g>

  <!-- ========================================== -->
  <!-- 05. BESPOKE SYSTEM & DOMAIN GLYPHS (REPLACING EMOJIS) -->
  <!-- ========================================== -->
  <g id="05. BESPOKE SYSTEM &amp; DOMAIN GLYPHS">
    <rect x="60" y="1275" width="1800" height="255" rx="12" fill="#18202C" stroke="#253549" stroke-width="1.5"/>
    <text x="84" y="1305" font-family="Inter, sans-serif" font-size="14" font-weight="800" fill="#DDE6ED">05. BESPOKE SYSTEM &amp; DOMAIN GLYPHS // 100% VECTOR EMOJI REPLACEMENT</text>
    <text x="84" y="1325" font-family="Inter, sans-serif" font-size="11" font-weight="500" fill="#9DB2BF">Strict 24×24 Pixel Grid // stroke="currentColor" // Zero OS Emoji Dependencies</text>

    <!-- 1. Dashboard -->
    <g transform="translate(90, 1350)">
      <rect width="64" height="64" rx="12" fill="#131B26" stroke="#3A4D63" stroke-width="1.2"/>
      <g transform="translate(18, 14) scale(1.2)" stroke="#38BDF8" fill="none" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
        <path d="M3 3v18h18"/><rect x="7" y="10" width="3" height="8" rx="1"/><rect x="13" y="6" width="3" height="12" rx="1"/><rect x="19" y="13" width="3" height="5" rx="1"/>
      </g>
      <text x="32" y="90" font-family="Inter, sans-serif" font-size="10" font-weight="600" fill="#DDE6ED" text-anchor="middle">Dashboard</text>
      <text x="32" y="104" font-family="Inter, sans-serif" font-size="9" fill="#526D82" text-anchor="middle">repl. 📊</text>
    </g>

    <!-- 2. Marketplace -->
    <g transform="translate(190, 1350)">
      <rect width="64" height="64" rx="12" fill="#131B26" stroke="#3A4D63" stroke-width="1.2"/>
      <g transform="translate(18, 14) scale(1.2)" stroke="#38BDF8" fill="none" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="9" cy="20" r="1.5"/><circle cx="18" cy="20" r="1.5"/>
        <path d="M2.5 3h3l2.6 11.2a1.5 1.5 0 0 0 1.5 1.2h9.4a1.5 1.5 0 0 0 1.5-1.1L22 7H6.2"/>
      </g>
      <text x="32" y="90" font-family="Inter, sans-serif" font-size="10" font-weight="600" fill="#DDE6ED" text-anchor="middle">Marketplace</text>
      <text x="32" y="104" font-family="Inter, sans-serif" font-size="9" fill="#526D82" text-anchor="middle">repl. 🛒</text>
    </g>

    <!-- 3. Brief -->
    <g transform="translate(290, 1350)">
      <rect width="64" height="64" rx="12" fill="#131B26" stroke="#3A4D63" stroke-width="1.2"/>
      <g transform="translate(18, 14) scale(1.2)" stroke="#38BDF8" fill="none" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
        <rect x="4" y="5" width="16" height="16" rx="2"/>
        <path d="M9 3h6a1 1 0 0 1 1 1v2a1 1 0 0 1-1 1H9a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1Z"/>
        <line x1="8" y1="11" x2="16" y2="11"/><line x1="8" y1="15" x2="13" y2="15"/>
      </g>
      <text x="32" y="90" font-family="Inter, sans-serif" font-size="10" font-weight="600" fill="#DDE6ED" text-anchor="middle">Brief / Audit</text>
      <text x="32" y="104" font-family="Inter, sans-serif" font-size="9" fill="#526D82" text-anchor="middle">repl. 📋</text>
    </g>

    <!-- 4. Contract -->
    <g transform="translate(390, 1350)">
      <rect width="64" height="64" rx="12" fill="#131B26" stroke="#3A4D63" stroke-width="1.2"/>
      <g transform="translate(18, 14) scale(1.2)" stroke="#38BDF8" fill="none" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
        <path d="M19 19a2 2 0 0 1-2 2H6a3 3 0 0 1-3-3V6a3 3 0 0 1 3-3h10a2 2 0 0 1 2 2v14Z"/>
        <path d="M18 17h2a2 2 0 0 1 2 2v0a2 2 0 0 1-2 2h-3"/>
        <line x1="7" y1="8" x2="14" y2="8"/><line x1="7" y1="12" x2="14" y2="12"/><line x1="7" y1="16" x2="11" y2="16"/>
      </g>
      <text x="32" y="90" font-family="Inter, sans-serif" font-size="10" font-weight="600" fill="#DDE6ED" text-anchor="middle">SLA Contract</text>
      <text x="32" y="104" font-family="Inter, sans-serif" font-size="9" fill="#526D82" text-anchor="middle">repl. 📜</text>
    </g>

    <!-- 5. Certification -->
    <g transform="translate(490, 1350)">
      <rect width="64" height="64" rx="12" fill="#131B26" stroke="#3A4D63" stroke-width="1.2"/>
      <g transform="translate(18, 14) scale(1.2)" stroke="#38BDF8" fill="none" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
        <path d="M22 10 12 5 2 10l10 5 10-5Z"/>
        <path d="M6 12v5c0 2.5 3 4 6 4s6-1.5 6-4v-5"/>
        <line x1="22" y1="10" x2="22" y2="16"/>
      </g>
      <text x="32" y="90" font-family="Inter, sans-serif" font-size="10" font-weight="600" fill="#DDE6ED" text-anchor="middle">Certificate</text>
      <text x="32" y="104" font-family="Inter, sans-serif" font-size="9" fill="#526D82" text-anchor="middle">repl. 🎓</text>
    </g>

    <!-- 6. Domains -->
    <g transform="translate(590, 1350)">
      <rect width="64" height="64" rx="12" fill="#131B26" stroke="#3A4D63" stroke-width="1.2"/>
      <g transform="translate(18, 14) scale(1.2)" stroke="#38BDF8" fill="none" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="9"/><line x1="3" y1="12" x2="21" y2="12"/>
        <path d="M12 3a14.5 14.5 0 0 0 0 18"/><path d="M12 3a14.5 14.5 0 0 1 0 18"/>
      </g>
      <text x="32" y="90" font-family="Inter, sans-serif" font-size="10" font-weight="600" fill="#DDE6ED" text-anchor="middle">Domains/DNS</text>
      <text x="32" y="104" font-family="Inter, sans-serif" font-size="9" fill="#526D82" text-anchor="middle">repl. 🌐</text>
    </g>

    <!-- 7. Freelance -->
    <g transform="translate(690, 1350)">
      <rect width="64" height="64" rx="12" fill="#131B26" stroke="#3A4D63" stroke-width="1.2"/>
      <g transform="translate(18, 14) scale(1.2)" stroke="#38BDF8" fill="none" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
        <rect x="3" y="7" width="18" height="14" rx="2"/>
        <path d="M8 7V5a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><line x1="3" y1="12" x2="21" y2="12"/><circle cx="12" cy="12" r="1.5"/>
      </g>
      <text x="32" y="90" font-family="Inter, sans-serif" font-size="10" font-weight="600" fill="#DDE6ED" text-anchor="middle">Specialist</text>
      <text x="32" y="104" font-family="Inter, sans-serif" font-size="9" fill="#526D82" text-anchor="middle">repl. 💼</text>
    </g>

    <!-- 8. Users -->
    <g transform="translate(790, 1350)">
      <rect width="64" height="64" rx="12" fill="#131B26" stroke="#3A4D63" stroke-width="1.2"/>
      <g transform="translate(18, 14) scale(1.2)" stroke="#38BDF8" fill="none" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="9" cy="7" r="4"/><path d="M2 20c0-3.3 3.1-6 7-6s7 2.7 7 6"/>
        <path d="M16 3.5a3.5 3.5 0 0 1 0 7"/><path d="M19 14.5c2 .7 3 2.3 3 4.5"/>
      </g>
      <text x="32" y="90" font-family="Inter, sans-serif" font-size="10" font-weight="600" fill="#DDE6ED" text-anchor="middle">Accounts</text>
      <text x="32" y="104" font-family="Inter, sans-serif" font-size="9" fill="#526D82" text-anchor="middle">repl. 👥</text>
    </g>

    <!-- 9. Package -->
    <g transform="translate(890, 1350)">
      <rect width="64" height="64" rx="12" fill="#131B26" stroke="#3A4D63" stroke-width="1.2"/>
      <g transform="translate(18, 14) scale(1.2)" stroke="#38BDF8" fill="none" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
        <path d="M12 3 3.5 7.8v8.4L12 21l8.5-4.8V7.8L12 3Z"/>
        <line x1="12" y1="3" x2="12" y2="21"/><line x1="3.5" y1="7.8" x2="12" y2="12.5"/><line x1="20.5" y1="7.8" x2="12" y2="12.5"/>
      </g>
      <text x="32" y="90" font-family="Inter, sans-serif" font-size="10" font-weight="600" fill="#DDE6ED" text-anchor="middle">Package .pr</text>
      <text x="32" y="104" font-family="Inter, sans-serif" font-size="9" fill="#526D82" text-anchor="middle">repl. 📦</text>
    </g>

    <!-- 10. Hardware -->
    <g transform="translate(990, 1350)">
      <rect width="64" height="64" rx="12" fill="#131B26" stroke="#3A4D63" stroke-width="1.2"/>
      <g transform="translate(18, 14) scale(1.2)" stroke="#38BDF8" fill="none" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
        <rect x="4" y="9" width="16" height="11" rx="2"/>
        <path d="M7 9V4a1 1 0 0 1 1-1h8a1 1 0 0 1 1 1v5"/><line x1="8" y1="14" x2="16" y2="14"/>
      </g>
      <text x="32" y="90" font-family="Inter, sans-serif" font-size="10" font-weight="600" fill="#DDE6ED" text-anchor="middle">POS Printer</text>
      <text x="32" y="104" font-family="Inter, sans-serif" font-size="9" fill="#526D82" text-anchor="middle">repl. 🖨</text>
    </g>

    <!-- 11. Search -->
    <g transform="translate(1090, 1350)">
      <rect width="64" height="64" rx="12" fill="#131B26" stroke="#3A4D63" stroke-width="1.2"/>
      <g transform="translate(18, 14) scale(1.2)" stroke="#38BDF8" fill="none" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="11" cy="11" r="7"/><line x1="16.5" y1="16.5" x2="21" y2="21"/>
      </g>
      <text x="32" y="90" font-family="Inter, sans-serif" font-size="10" font-weight="600" fill="#DDE6ED" text-anchor="middle">Spotlight</text>
      <text x="32" y="104" font-family="Inter, sans-serif" font-size="9" fill="#526D82" text-anchor="middle">repl. 🔍</text>
    </g>

    <!-- 12. Settings -->
    <g transform="translate(1190, 1350)">
      <rect width="64" height="64" rx="12" fill="#131B26" stroke="#3A4D63" stroke-width="1.2"/>
      <g transform="translate(18, 14) scale(1.2)" stroke="#38BDF8" fill="none" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="3.5"/>
        <path d="M12 2v2.5M12 19.5V22M2 12h2.5M19.5 12H22M4.9 4.9l1.8 1.8M17.3 17.3l1.8 1.8M4.9 19.1l1.8-1.8M17.3 6.7l1.8-1.8"/>
      </g>
      <text x="32" y="90" font-family="Inter, sans-serif" font-size="10" font-weight="600" fill="#DDE6ED" text-anchor="middle">Engine Config</text>
      <text x="32" y="104" font-family="Inter, sans-serif" font-size="9" fill="#526D82" text-anchor="middle">repl. ⚙</text>
    </g>

    <!-- 13. Lightning -->
    <g transform="translate(1290, 1350)">
      <rect width="64" height="64" rx="12" fill="#131B26" stroke="#3A4D63" stroke-width="1.2"/>
      <g transform="translate(18, 14) scale(1.2)" stroke="#38BDF8" fill="none" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
        <polygon points="13 2 4 14 11 14 9 22 20 10 13 10 13 2"/>
      </g>
      <text x="32" y="90" font-family="Inter, sans-serif" font-size="10" font-weight="600" fill="#DDE6ED" text-anchor="middle">Sync Pulse</text>
      <text x="32" y="104" font-family="Inter, sans-serif" font-size="9" fill="#526D82" text-anchor="middle">repl. ⚡</text>
    </g>
  </g>

  <!-- ========================================== -->
  <!-- 06. HARDWARE POS & FIELD ENGINE EMBLEMS -->
  <!-- ========================================== -->
  <g id="06. HARDWARE SPOOLER &amp; FIELD ENGINE EMBLEMS">
    <rect x="60" y="1550" width="1800" height="235" rx="12" fill="#18202C" stroke="#253549" stroke-width="1.5"/>
    <text x="84" y="1580" font-family="Inter, sans-serif" font-size="14" font-weight="800" fill="#DDE6ED">06. HARDWARE SPOOLER &amp; FIELD ENGINE EMBLEMS</text>

    <!-- ESC/POS Thermal 80mm -->
    <g transform="translate(100, 1605)">
      <rect width="180" height="150" rx="12" fill="#1E2A3A" stroke="#526D82" stroke-width="1.5"/>
      <rect x="45" y="35" width="90" height="70" rx="8" fill="#27374D" stroke="#DDE6ED" stroke-width="3"/>
      <path d="M 60 35 L 60 18 L 120 18 L 120 35" fill="none" stroke="#9DB2BF" stroke-width="3"/>
      <line x1="60" y1="60" x2="120" y2="60" stroke="#DDE6ED" stroke-width="3.5" stroke-linecap="round"/>
      <circle cx="118" cy="75" r="3" fill="#34D399"/>
      <text x="90" y="132" font-family="Inter, sans-serif" font-size="12" font-weight="700" fill="#DDE6ED" text-anchor="middle">ESC/POS 80mm Spooler</text>
    </g>

    <!-- Cash Drawer Kick -->
    <g transform="translate(320, 1605)">
      <rect width="180" height="150" rx="12" fill="#1E2A3A" stroke="#526D82" stroke-width="1.5"/>
      <rect x="35" y="35" width="110" height="65" rx="6" fill="#27374D" stroke="#DDE6ED" stroke-width="3"/>
      <line x1="35" y1="55" x2="145" y2="55" stroke="#526D82" stroke-width="2.5"/>
      <rect x="75" y="65" width="30" height="8" rx="3" fill="#526D82"/>
      <circle cx="90" cy="45" r="3" fill="#DDE6ED"/>
      <text x="90" y="132" font-family="Inter, sans-serif" font-size="12" font-weight="700" fill="#DDE6ED" text-anchor="middle">Cash Drawer Kick</text>
    </g>

    <!-- Barcode Scanner -->
    <g transform="translate(540, 1605)">
      <rect width="180" height="150" rx="12" fill="#1E2A3A" stroke="#526D82" stroke-width="1.5"/>
      <path d="M 60 30 L 105 30 C 110 30 115 35 115 40 L 115 55 L 90 85 L 80 85 L 75 55 L 60 55 L 60 35 C 60 32 63 30 66 30 Z" fill="#27374D" stroke="#DDE6ED" stroke-width="3"/>
      <line x1="40" y1="42" x2="135" y2="42" stroke="#EF4444" stroke-width="3.5" stroke-linecap="round"/>
      <text x="90" y="132" font-family="Inter, sans-serif" font-size="12" font-weight="700" fill="#DDE6ED" text-anchor="middle">GS1-128 Scanner</text>
    </g>

    <!-- LAN Discovery UDP Beacon -->
    <g transform="translate(760, 1605)">
      <rect width="180" height="150" rx="12" fill="#1E2A3A" stroke="#526D82" stroke-width="1.5"/>
      <circle cx="90" cy="55" r="14" fill="#27374D" stroke="#DDE6ED" stroke-width="3"/>
      <circle cx="90" cy="55" r="28" fill="none" stroke="#9DB2BF" stroke-width="2.5" stroke-dasharray="4 4"/>
      <circle cx="90" cy="55" r="42" fill="none" stroke="#526D82" stroke-width="2" stroke-dasharray="6 6"/>
      <text x="90" y="132" font-family="Inter, sans-serif" font-size="12" font-weight="700" fill="#DDE6ED" text-anchor="middle">UDP Beacon :7444</text>
    </g>
  </g>
</svg>"""

# Validate SVG XML before writing
ET.fromstring(master_svg)

with open(MASTER_SVG_PATH, "w", encoding="utf-8") as f:
    f.write(master_svg)

print(f"Updated Master SVG artboard at {MASTER_SVG_PATH} (1920x1820) with XML validation OK")
