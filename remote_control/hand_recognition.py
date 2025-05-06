import json
import sys
import cv2
import mediapipe as mp
import numpy as np

def extract_landmarks_get_pixels(frame, hand_landmarks):

    # Extract landmarks
    h, w, _ = frame.shape
    wrist = hand_landmarks.landmark[0]       # Wrist
    fingertip = hand_landmarks.landmark[8]   # Index fingertip
    
    # Convert to pixel coordinates
    x0, y0 = int(wrist.x * w), int(wrist.y * h)
    x1, y1 = int(fingertip.x * w), int(fingertip.y * h)
    
    return (x0, x1, y0, y1)

def compute_angle(x0, x1, y0, y1):
    
    # Compute angle (0 = up, 90 = left/right, 180 = down)
    dx = x0 - x1    # inverted X axis when since looking at Right hand
    dy = y0 - y1    # inverted Y axis for screen coords
    
    # dx/dy since finding angle with vertical axis
    angle_rad = np.arctan2(dx, dy)
    
    angle_deg = (np.degrees(angle_rad) + 360) % 360

    # Normalize to [0, 180]
    if angle_deg > 270 and angle_deg <= 360:
        angle_deg = 0
    elif angle_deg > 180 and angle_deg <= 270:
        angle_deg = 180
    
    # Apply smoothing of angles to remove noise
    angle_deg = (angle_deg // 10) * 10
    
    return angle_deg

def stop_or_go(hand_landmarks):
    # Finger   |   Tip ID   |   Middle ID
    # ------------------------------------
    # Index    |   8        |   6
    # Middle   |   12       |   10
    # Ring     |   16       |   14
    # Pinky    |   20       |   18
    
    tips = [8, 12, 16, 20]
    middles = [6, 10, 14, 18]
    
    extended_fingers = 0
    
    for tip_id, middle_id in zip(tips, middles):
        tip = hand_landmarks.landmark[tip_id]
        middle = hand_landmarks.landmark[middle_id]
        
        if tip.y < middle.y:
            extended_fingers += 1
    
    result = "STOP" if extended_fingers < 3 else "GO"
        
    return result

def display_info(stop_or_go, angle_deg, frame):    
    text = f"{stop_or_go}, Angle: {int(angle_deg)} degrees"
    cv2.putText(frame, text, (10, 50),
                cv2.FONT_HERSHEY_SIMPLEX, 1, (0, 0, 255), 2)

def main():
    # Initialize MediaPipe Hands
    mp_drawing = mp.solutions.drawing_utils
    mp_hands = mp.solutions.hands

    # Start video capture
    cap = cv2.VideoCapture(0)
    
    # Car will initially point straight and be stationary
    angle_deg = 90
    pedal_or_break = "STOP"

    with mp_hands.Hands(
        max_num_hands=1,
        min_detection_confidence=0.7,
        min_tracking_confidence=0.5
    ) as hands:

        while cap.isOpened():
            success, frame = cap.read()
            if not success:
                break

            # Flip image for natural selfie view and convert BGR to RGB
            frame = cv2.flip(frame, 1)
            rgb = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)

            # Get hand results from model
            results = hands.process(rgb)

            if results.multi_hand_landmarks and results.multi_handedness:
                for hand_landmarks, handedness in zip(results.multi_hand_landmarks, results.multi_handedness):
                    # Draw hand landmarks
                    mp_drawing.draw_landmarks(frame, hand_landmarks, mp_hands.HAND_CONNECTIONS)
                    
                    # Get label: "Left" or "Right"
                    hand_label = handedness.classification[0].label

                    # Extract hand landmarks and convert to pixel coordinates
                    x0, x1, y0, y1 = extract_landmarks_get_pixels(frame, hand_landmarks)

                    # Draw vector line from wrist to fingertip
                    cv2.line(frame, (x0, y0), (x1, y1), (0, 255, 0), 3)
                    cv2.circle(frame, (x1, y1), 6, (0, 0, 255), -1)
                    
                    # Compute angle between fingertip and wrist
                    if hand_label == "Right":
                        angle_deg = compute_angle(x0, x1, y0, y1)

                    # Determine whether car should stop or go
                    if hand_label == "Left":    
                        pedal_or_break = stop_or_go(hand_landmarks)
                    
                    # Display information on video
                    display_info(pedal_or_break, angle_deg, frame)

                    # print data as JSON in console for server to read
                    print(json.dumps({
                        "state": pedal_or_break,
                        "angle": int(angle_deg)
                    }))
                    sys.stdout.flush()
            
            # Show frame
            cv2.imshow('Finger Angle Tracker', frame)
            if cv2.waitKey(1) & 0xFF == 27:
                break  # Press ESC to exit

    cap.release()
    cv2.destroyAllWindows()

if __name__ == "__main__":
    main()