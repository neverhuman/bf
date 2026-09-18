import {defineConfig} from "@playwright/test";
export default defineConfig({testDir:"./browser",fullyParallel:false,workers:1,timeout:30000,use:{headless:true,viewport:{width:1000,height:800}},reporter:"list"});
