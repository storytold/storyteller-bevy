import fs from "fs";
import path from "path";
import { ViteDevServer } from "vite";

interface IPropsFile {
    name: string;
    files: string[];
}

function getFiles(dir, files_): string[] {
    files_ = files_ || [];
    const files = fs.readdirSync(dir);
    for (const i in files) {
        const name = dir + "/" + files[i];
        if (fs.statSync(name).isDirectory()) {
            getFiles(name, files_);
        } else {
            files_.push(name);
        }
    }
    return files_;
}

const mimeTypes = {
    ".7z": "application/x-7z-compressed",
    ".bin": "application/octet-stream",
    ".bvh": "text/bvh",
    ".css": "text/css",
    ".eot": "font/eot",
    ".gif": "image/gif",
    ".gltf": "model/gltf+json",
    ".gz": "application/gzip",
    ".hdr": "image/vnd.radiance",
    ".html": "text/html",
    ".ico": "image/x-icon",
    ".jpg": "image/jpeg",
    ".js": "text/javascript",
    ".json": "application/json",
    ".ktx": "image/ktx",
    ".ktx2": "image/ktx",
    ".meta": "text/plain",
    ".mjs": "text/javascript",
    ".mp3": "audio/mpeg",
    ".mp4": "video/mp4",
    ".otf": "font/otf",
    ".png": "image/png",
    ".rar": "application/x-rar-compressed",
    ".svg": "image/svg+xml",
    ".tiff": "image/tiff",
    ".ttf": "font/ttf",
    ".txt": "text/plain",
    ".wasm": "application/wasm",
    ".webm": "video/webm",
    ".webp": "image/webp",
    ".wgsl": "text/wgsl",
    ".woff": "font/woff",
    ".woff2": "font/woff2",
    ".xml": "text/xml",
    ".zip": "application/zip",
}

export function ServerMiddleWare(server: ViteDevServer, assets: string[] = []) {
    if (!assets || !assets.length)
        return;

    const fileObject: IPropsFile[] = [];
    for (let i = 0; i < assets.length; i++) {
        const files = getFiles(path.join(process.cwd(), `/${assets[i]}`), []);
        fileObject.push({
            name: assets[i],
            files
        });
    }

    return () => {
        server.middlewares.use(async (req, res, next) => {
            for (let i = 0; i < fileObject.length; i++) {
                const file = path.join(process.cwd(), `${fileObject[i].name}/${req.originalUrl}`);
                if (fileObject[i].files.some(f => path.relative(f, file) === "")) {
                    const extension = file.substring(file.lastIndexOf("."));
                    res.setHeader("Cache-Control", "max-age=31536000, immutable");
                    res.setHeader("Content-Type", mimeTypes[extension]);
                    res.writeHead(200);
                    res.write(fs.readFileSync(path.join(process.cwd(), "/" + fileObject[i].name + req.originalUrl)));
                    res.end();
                    break;
                }
            }
            next();
        });
    };
}
