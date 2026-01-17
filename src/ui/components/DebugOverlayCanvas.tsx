import { useEffect, useRef } from "react";
import type { FrameEvent } from "../../cv/types";

type DebugOverlayCanvasProps = {
  frame?: FrameEvent;
};

export const DebugOverlayCanvas = ({ frame }: DebugOverlayCanvasProps) => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  useEffect(() => {
    if (!frame) {
      return;
    }
    const canvas = canvasRef.current;
    if (!canvas) {
      return;
    }

    canvas.width = frame.frameWidth;
    canvas.height = frame.frameHeight;
    const ctx = canvas.getContext("2d");
    if (!ctx) {
      return;
    }

    const draw = async () => {
      ctx.clearRect(0, 0, canvas.width, canvas.height);

      if (frame.image && frame.image.length > 0) {
        try {
          const blob = new Blob([new Uint8Array(frame.image)], { type: "image/jpeg" });
          const bitmap = await createImageBitmap(blob);
          ctx.drawImage(bitmap, 0, 0, canvas.width, canvas.height);
          bitmap.close();
        } catch (e) {
          console.error("Failed to draw frame image:", e);
        }
      }
      
      // Calculate scale factor based on frame width, assuming ~640px is "normal"
      const scale = Math.max(1, frame.frameWidth / 640);
      const fontSize = Math.floor(14 * scale);
      const padding = 4 * scale;
      const lineHeight = Math.floor(20 * scale);
      
      ctx.font = `${fontSize}px sans-serif`;
      ctx.lineWidth = 2 * scale;

      frame.faces.forEach((face) => {
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
    };

    draw();
  }, [frame]);

  return <canvas ref={canvasRef} className="debug-canvas" />;
};
