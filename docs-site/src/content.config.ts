import { defineCollection } from 'astro:content'
import { docsSchema } from '@astrojs/starlight/schema'
import { glob } from 'astro/loaders'

const docs = defineCollection({
  loader: glob({
    pattern: ['*.md', 'user/*.md', 'baseline/**/*.md', 'runbooks/*.md'],
    base: '../docs',
  }),
  schema: docsSchema(),
})

export const collections = { docs }
