export const extractColors = async (imgUrl: string, options: any) => {
	const img = await loadImage(imgUrl, options);
	const imageData = getImageData(img);
	return calculateColors(imageData);
};

const loadImage = (imgUrl: string, options: any): Promise<HTMLImageElement> => {
	return new Promise((resolve, reject) => {
		const img = new Image();
		img.crossOrigin = options.crossOrigin;
		img.src = imgUrl;
		img.onload = () => resolve(img);
		img.onerror = (e) => reject(e);
	});
};

const getImageData = (img: HTMLImageElement) => {
	const canvas = document.createElement('canvas');
	canvas.width = img.width;
	canvas.height = img.height;
	const ctx = canvas.getContext('2d');
	if (!ctx) {
		throw new Error('Failed to get 2D context');
	}
	ctx.drawImage(img, 0, 0);
	return ctx.getImageData(0, 0, canvas.width, canvas.height);
};

const calculateColors = (imageData: ImageData | undefined) => {
	if (!imageData) return [];
	const colors = [];
	for (let i = 0; i < imageData.data.length; i += 4) {
		const [r, g, b, a] = [
			imageData.data[i],
			imageData.data[i + 1],
			imageData.data[i + 2],
			imageData.data[i + 3]
		];

		if (a === 0) continue;

		const hsl = rgbToHsl(r, g, b);
		colors.push({
			rgb: { r, g, b },
			hsl,
			hex: rgbToHex(r, g, b),
			lightness: hsl[2]
		});
	}
	return colors;
};

const rgbToHsl = (r: number, g: number, b: number) => {
	r /= 255;
	g /= 255;
	b /= 255;
	const max = Math.max(r, g, b);
	const min = Math.min(r, g, b);
	let h = 0;
	let s = 0;
	const l = (max + min) / 2;

	if (max !== min) {
		const d = max - min;
		s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
		switch (max) {
			case r:
				h = (g - b) / d + (g < b ? 6 : 0);
				break;
			case g:
				h = (b - r) / d + 2;
				break;
			case b:
				h = (r - g) / d + 4;
				break;
		}
		h /= 6;
	}
	return [h, s, l];
};

const rgbToHex = (r: number, g: number, b: number) => {
	return `#${((1 << 24) + (r << 16) + (g << 8) + b).toString(16).slice(1)}`;
};
