module.exports = {
    content: [
        "./frontend/src/**/*.rs",
        "./frontend/public/**/*.{html,js}",
    ],
    theme: {
        extend: {
            margin: {
                '1.5px': '1.5px',
            },
            fontFamily: {
                'system': ["-apple-system, BlinkMacSystemFont, avenir next, avenir, segoe ui, helvetica neue, helvetica, Ubuntu, roboto, noto, arial, sans-serif"]
            },
            fontSize: {
                "2xlplus": ["1.6rem", "2.13rem"]
            },
            transitionProperty: {
                'height': 'height',
            },
            colors: {
                "blue_400_saturated": "#65D6FC",
                "gray150": "#ECEEF1",
            }
        }
    },
};
