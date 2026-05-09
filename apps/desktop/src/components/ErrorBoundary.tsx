import { Component, type ErrorInfo, type ReactNode } from "react";

type ErrorBoundaryProps = {
  children: ReactNode;
  title: string;
};

type ErrorBoundaryState = {
  message: string | null;
};

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  state: ErrorBoundaryState = {
    message: null,
  };

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { message: error.message };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error(error, info.componentStack);
  }

  render() {
    if (this.state.message) {
      return (
        <section className="panel error-panel">
          <h3>{this.props.title}</h3>
          <p>{this.state.message}</p>
        </section>
      );
    }

    return this.props.children;
  }
}
