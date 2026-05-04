import { useCallback, useEffect, useRef } from "react";
import type { FrameEvent } from "../../cv/types";

type DebugOverlayCanvasProps = {
  frame?: FrameEvent;
};

export const DebugOverlayCanvas = ({ frame }: DebugOverlayCanvasProps) => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const latestFrameRef = useRef<FrameEvent | undefined>(undefined);
  const drawingRef = useRef(false);

  const drawFrame = useCallback(async (nextFrame: FrameEvent) => {
    const canvas = canvasRef.current;
    if (!canvas) {
      return;
    }

    canvas.width = nextFrame.frameWidth;
    canvas.height = nextFrame.frameHeight;
    const ctx = canvas.getContext("2d");
    if (!ctx) {
      return;
    }

    let bitmap: ImageBitmap | undefined;
    if (nextFrame.image && nextFrame.image.length > 0) {
      const blob = new Blob([new Uint8Array(nextFrame.image)], { type: "image/jpeg" });
      bitmap = await createImageBitmap(blob);
    }

    ctx.clearRect(0, 0, canvas.width, canvas.height);

    if (bitmap) {
      ctx.drawImage(bitmap, 0, 0, canvas.width, canvas.height);
      bitmap.close();
    }

    const scale = Math.max(1, nextFrame.frameWidth / 640);
    const fontSize = Math.floor(14 * scale);
    const padding = 4 * scale;
    const lineHeight = Math.floor(20 * scale);

    ctx.font = `${fontSize}px sans-serif`;
    ctx.lineWidth = 2 * scale;

    nextFrame.faces.forEach((face) => {
      ctx.strokeStyle = face.label === "owner" ? "#22c55e" : "#ef4444";
      ctx.strokeRect(face.bbox.x, face.bbox.y, face.bbox.width, face.bbox.height);

      const score = typeof face.observerScore === "number" ? ` ${face.observerScore.toFixed(2)}` : "";
      const text = `${face.label}${score}`;
      const textMetrics = ctx.measureText(text);
      const textWidth = textMetrics.width + (padding * 2);

      ctx.fillStyle = "rgba(0,0,0,0.6)";
      ctx.fillRect(face.bbox.x, face.bbox.y - lineHeight, textWidth, lineHeight);

      ctx.fillStyle = "#ffffff";
      ctx.fillText(text, face.bbox.x + padding, face.bbox.y - (6 * scale));
    });
  }, []);

  useEffect(() => {
    if (!frame) {
      return;
    }
    latestFrameRef.current = frame;
    if (drawingRef.current) {
      return;
    }

    drawingRef.current = true;
    const drawLatest = async () => {
      while (latestFrameRef.current) {
        const nextFrame = latestFrameRef.current;
        latestFrameRef.current = undefined;
        await drawFrame(nextFrame);
      }
      drawingRef.current = false;
      if (latestFrameRef.current) {
        drawingRef.current = true;
        void drawLatest();
      }
    };

    void drawLatest().catch((error) => {
      console.error("Failed to draw debug overlay frame:", error);
      drawingRef.current = false;
    });
  }, [drawFrame, frame]);

  return <canvas ref={canvasRef} className="debug-canvas" />;
};
