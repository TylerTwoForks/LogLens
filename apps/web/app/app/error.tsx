"use client";

import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";

export default function AppError({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  return (
    <Card className="max-w-lg border-destructive">
      <CardHeader>
        <CardTitle className="text-destructive">
          Something went wrong
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <p className="text-sm text-muted-foreground">{error.message}</p>
        <Button variant="outline" onClick={reset}>
          Try again
        </Button>
      </CardContent>
    </Card>
  );
}
