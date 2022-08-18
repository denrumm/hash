import { FunctionComponent } from "react";
import { SvgIcon, SvgIconProps } from "@mui/material";

export const TwitterIcon: FunctionComponent<SvgIconProps> = ({
  sx = [],
  ...props
}) => (
  <SvgIcon
    {...props}
    viewBox="0 0 20 17"
    sx={[...(Array.isArray(sx) ? sx : [sx]), { height: "1em" }]}
  >
    <path
      d="M17.9442 4.4242C17.9569 4.60185 17.9569 4.77955 17.9569 4.95721C17.9569 10.376 13.8326 16.6197 6.29444 16.6197C3.97209 16.6197 1.81473 15.9471 0 14.7796C0.329962 14.8176 0.64719 14.8303 0.989848 14.8303C2.90607 14.8303 4.67006 14.1831 6.0787 13.0791C4.27666 13.041 2.7665 11.8608 2.24618 10.2364C2.50001 10.2745 2.7538 10.2998 3.02032 10.2998C3.38833 10.2998 3.75638 10.2491 4.099 10.1603C2.22083 9.77953 0.812152 8.1298 0.812152 6.13741V6.08666C1.35782 6.39123 1.99239 6.58159 2.66493 6.60694C1.56087 5.87088 0.837542 4.61455 0.837542 3.19321C0.837542 2.43181 1.04055 1.73383 1.3959 1.12469C3.41369 3.612 6.4467 5.23635 9.8477 5.41404C9.78426 5.10947 9.74617 4.79224 9.74617 4.47498C9.74617 2.21606 11.5736 0.375977 13.8452 0.375977C15.0254 0.375977 16.0914 0.870901 16.8401 1.6704C17.7665 1.49274 18.6548 1.15008 19.4416 0.680548C19.137 1.63235 18.4898 2.43184 17.6396 2.93942C18.4645 2.85064 19.264 2.62216 20 2.30493C19.4417 3.11708 18.7437 3.8404 17.9442 4.4242Z"
      fill="currentColor"
    />
  </SvgIcon>
);
