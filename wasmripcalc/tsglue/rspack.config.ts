import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    ripcalc: './src/ripcalc.ts',
  },
  module: {
    rules: [
      {
        test: /\.ts$/,
        exclude: [/node_modules/],
        loader: 'builtin:swc-loader',
        options: {
          detectSyntax: 'auto',
          jsc: {
            target: 'es3',
          },
        },
      },
    ],
  },
  devtool: 'source-map',
});
