const fs = require('fs');
const path = require('path');

const file = path.join(__dirname, '../src/readonly.rs');
const readme = path.join(__dirname, '../README.md');

const readmeContent = fs.readFileSync(readme, 'utf8');
const fileContent = fs.readFileSync(file, 'utf8');

let help = fileContent.match(/help\(\)((.|\n)*)\}\n/)[0].match(/"((.|\n)*)"/)[0].replace(/"/g, '\n');

let index_start = readmeContent.indexOf('Documentation') + 'Documentation'.length;
let index_end = readmeContent.indexOf('## Authors');

let newReadme = readmeContent.slice(0, index_start) + help + readmeContent.slice(index_end);
fs.writeFileSync(readme, newReadme);