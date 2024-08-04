// simple hello world example in nodejs

const fs = require('fs')

let article = fs.readFileSync('../public/article0.html', 'utf8')
let full_article = fs.readFileSync('../public/full/article0.html', 'utf8')
let index = fs.readFileSync('src/index.js', 'utf8')

article = article.replace(/\n/g, '')
full_article = full_article.replace(/\n/g, '')

index = index.replace(/const ARTICLE = `[^`]*`/, `const ARTICLE = \`${article}\``)
index = index.replace(/const FULL_ARTICLE = `[^`]*`/, `const FULL_ARTICLE = \`${full_article}\``)

fs.writeFileSync('src/index.js', index)

