# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
project = 'cubic'
copyright = '2025, Cubic'
author = 'Roger Knecht'
release = 'v0.0.0-dev'

extensions = ['sphinx_rtd_theme']

templates_path = ['_templates']
exclude_patterns = []

html_theme = 'sphinx_rtd_theme'
html_theme_options = {
    'navigation_depth': 1,
    'collapse_navigation': False
}
html_sidebars = { '**': ['globaltoc.html', 'searchbox.html'] }
html_static_path = ['_static']
html_logo = "logo.svg"
