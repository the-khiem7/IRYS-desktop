import { defineCollection } from 'astro:content';
import { glob } from 'astro/loaders';
import { docsSchema } from '@astrojs/starlight/schema';

const docs = defineCollection({
  loader: glob({
    pattern: ['*.md', 'user/*.md', 'baseline/**/*.md', 'runbooks/*.md'],
    base: '../docs',
  }),
  schema: docsSchema(),
});

export const collections = { docs };
