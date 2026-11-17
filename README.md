<div align="center">
  <h1>DropTidy v0.1.0</h1>
  <p><strong>A fast, open-source Windows Downloads organizer built with Rust, Tauri v2, and React.</strong></p>
  <p>
    <img src="https://shields.io" alt="Windows" />
    <img src="https://shields.io" alt="Tauri v2" />
    <img src="https://shields.io" alt="License" />
  </p>
</div>

<hr />

<h2>Key Features</h2>
<ul>
  <li><strong>Safe Dry-Run Preview:</strong> Review every file, size, category, and exact destination path before moving anything. Toggle individual files or entire categories on or off.</li>
  <li><strong>Smart Extension Categorization:</strong> Automatically groups top-level files into <code>Installers</code>, <code>Archives</code>, <code>Documents</code>, <code>Images</code>, <code>Media</code>, <code>Code</code>, and <code>Other</code> while ignoring subfolders and active browser downloads (<code>.crdownload</code>, <code>.part</code>, <code>.tmp</code>, <code>.opdownload</code>).</li>
  <li><strong>SHA-256 Duplicate Detection:</strong> Uses two-stage partial-to-full SHA-256 hashing to flag identical downloads right in the preview table.</li>
  <li><strong>One-Click SQLite Undo:</strong> Every sort batch is logged locally in SQLite. Click <strong>Undo Last Sort</strong> anytime to restore files to their original paths and clean up empty category folders.</li>
  <li><strong>Auto-Organize Watch Mode and System Tray:</strong> Optional background watcher waits for downloads to finish saving and unlocks before sorting automatically, with quick controls in the Windows system tray.</li>
  <li><strong>Zero Overwrites:</strong> Safely appends <code>(1)</code>, <code>(2)</code>, etc. on filename collisions and supports cross-drive moves.</li>
</ul>

<h2>Installation</h2>
<ol>
  <li>Download <code>DropTidy_0.1.0_x64-setup.exe</code> from the <strong>Assets</strong> section below.</li>
  <li>Run the installer. Because this is an unsigned open-source release, Windows SmartScreen may prompt you on first run. Click <strong>More info</strong> and then <strong>Run anyway</strong>.</li>
</ol>
