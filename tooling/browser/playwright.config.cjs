const {defineConfig} = require('@playwright/test');
module.exports = defineConfig({
  testDir: './tests', workers:1, fullyParallel:false,
  use: {baseURL:'http://127.0.0.1:5191',trace:'retain-on-failure'},
  webServer: {command:'python3 ../../scripts/browser_server.py',url:'http://127.0.0.1:5191/api/ready',reuseExistingServer:false,timeout:30000},
});
