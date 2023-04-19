module.exports = {
	root: true,
	env: {
		es6: true,
		node: true,
	},
	extends: [
		'eslint:recommended',
		'plugin:import/errors',
		'plugin:import/warnings',
		'plugin:import/typescript',
		'google',
		'plugin:@typescript-eslint/recommended',
	],
	parser: '@typescript-eslint/parser',
	parserOptions: {
		project: ['tsconfig.json', 'tsconfig.dev.json'],
		sourceType: 'module',
	},
	ignorePatterns: [
		'/lib/**/*', // Ignore built files.
	],
	plugins: [
		'@typescript-eslint',
		'import',
	],
	rules: {
		'quotes': ['error', 'single'],
		'indent': ['error', 'tab'],
		'no-tabs': ['error', { allowIndentationTabs: true }],
		'object-curly-spacing': ['error', 'always'],
		'arrow-parens': ['error', 'as-needed'],
		'import/no-unresolved': 0,
	},
};