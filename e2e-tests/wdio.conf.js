import os from 'os';
import path from 'path';
import { spawn, spawnSync } from 'child_process';
import { fileURLToPath } from 'url';

const __dirname = fileURLToPath(new URL('.', import.meta.url));

// Keep track of the `tauri-driver` child process
let tauriDriver;
let exit = false;

export const config = {
  host: '127.0.0.1',
  port: 4444,
  specs: ['./test/specs/**/*.js'],
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      'tauri:options': {
        application: '../src-tauri/target/debug/colimail.exe',
      },
    },
  ],
  reporters: ['spec'],
  framework: 'mocha',
  mochaOpts: {
    ui: 'bdd',
    timeout: 60000,
  },

  // Ensure the Rust project is built since we expect this binary to exist for the WebDriver sessions
  onPrepare: () => {
    console.log('Building Tauri app in debug mode...');
    spawnSync('npm', ['run', 'tauri', 'build', '--', '--debug'], {
      cwd: path.resolve(__dirname, '..'),
      stdio: 'inherit',
      shell: true,
    });
  },

  // Ensure we are running `tauri-driver` before the session starts so that we can proxy the WebDriver requests
  beforeSession: () => {
    console.log('Starting tauri-driver...');
    const tauriDriverPath = path.resolve(
      os.homedir(),
      '.cargo',
      'bin',
      process.platform === 'win32' ? 'tauri-driver.exe' : 'tauri-driver'
    );

    tauriDriver = spawn(tauriDriverPath, [], {
      stdio: [null, process.stdout, process.stderr],
    });

    tauriDriver.on('error', (error) => {
      console.error('tauri-driver error:', error);
      process.exit(1);
    });

    tauriDriver.on('exit', (code) => {
      if (!exit) {
        console.error('tauri-driver exited with code:', code);
        process.exit(1);
      }
    });
  },

  // Clean up the `tauri-driver` process we spawned at the start of the session
  afterSession: () => {
    closeTauriDriver();
  },
};

function closeTauriDriver() {
  exit = true;
  tauriDriver?.kill();
}

function onShutdown(fn) {
  const cleanup = () => {
    try {
      fn();
    } finally {
      process.exit();
    }
  };

  process.on('exit', cleanup);
  process.on('SIGINT', cleanup);
  process.on('SIGTERM', cleanup);
  process.on('SIGHUP', cleanup);
  process.on('SIGBREAK', cleanup);
}

// Ensure tauri-driver is closed when our test process exits
onShutdown(() => {
  closeTauriDriver();
});
